use std::path::Path;

use crate::common::{
    CastFile, build_system_prompt, fm_list, fm_str, merge_json_objects, read_all_rules,
    read_existing_json_map, read_skill_dir_files, yaml_frontmatter,
};
use crate::harness_config::CopilotConfig;
use anyhow::Result;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use theta_harness::layout::{CopilotLayout, HarnessLayout};
use theta_schema::{ApplyMode, Diagnostic, ThetaManifest};

// orchestrator

/// Shared body for `cast_files` and `cast_files_with_output`.
pub(super) fn cast_files_internal(
    manifest: &ThetaManifest,
    theta_dir: &Path,
    output_dir: Option<&Path>,
) -> Result<Vec<CastFile>> {
    let mut files = Vec::new();

    let key = theta_harness::HarnessTarget::Copilot.toml_key();
    let cfg: Option<CopilotConfig> = manifest
        .harness_config(key)
        .map_err(|e| anyhow::anyhow!("[harness.{key}]: failed to parse: {e}"))?;

    if let Some(content) = build_system_prompt(manifest, theta_dir)? {
        files.push((CopilotLayout::system_prompt(), content.into()));
    }

    for (name, rule, content) in read_all_rules(manifest, theta_dir)? {
        let mut fm: Vec<(&str, serde_norway::Value)> = Vec::new();

        if let Some(ref desc) = rule.description {
            fm.push(("description", fm_str(desc)));
        }
        match rule.apply {
            ApplyMode::Always => {
                // VS Code requires applyTo for auto-application; '**' = always-on
                fm.push(("applyTo", fm_str("**")));
            }
            ApplyMode::Glob => {
                if let Some(ref patterns) = rule.apply_to {
                    fm.push(("applyTo", fm_str(patterns.join(", "))));
                }
            }
            _ => {}
        }

        let frontmatter = yaml_frontmatter(&fm)?;
        files.push((
            CopilotLayout::rule(&name),
            format!("{frontmatter}{content}").into(),
        ));
    }

    // [skills] --> .github/skills/<name>/
    // ref: https://code.visualstudio.com/docs/copilot/customization/agent-skills
    if let Some(ref skills) = manifest.skills {
        for name in skills.keys() {
            let skill_files = read_skill_dir_files(theta_dir, name, &CopilotLayout::skills_dir())?;
            files.extend(skill_files);
        }
    }

    // read existing shared VS Code files once so the merge is deterministic
    // and doesn't re-read per-section
    let existing_mcp = match output_dir {
        Some(d) => read_existing_json_map(&d.join(CopilotLayout::mcp()))?,
        None => None,
    };
    let existing_settings = match output_dir {
        Some(d) => read_existing_json_map(&d.join(CopilotLayout::settings()))?,
        None => None,
    };

    // [tools] + [harness.github_copilot.tool.<name>] --> .vscode/mcp.json
    files.extend(cast_mcp_file(
        manifest,
        cfg.as_ref(),
        existing_mcp.as_ref(),
    )?);

    // [[subagents]] + [harness.github_copilot.subagent.<name>] --> .github/agents/*.agent.md
    files.extend(cast_subagent_files(manifest, theta_dir, cfg.as_ref())?);

    // [harness.github_copilot] --> .vscode/settings.json
    files.extend(cast_settings_file(manifest, existing_settings.as_ref())?);

    // [harness.github_copilot].hooks --> .github/hooks/theta-hooks.json
    // theta manages the hook config (event --> command mapping) but NOT the
    // scripts those commands reference - same as MCP server binaries.
    files.extend(cast_hooks_file(cfg.as_ref())?);

    Ok(files)
}

/// Build the theta-owned `servers` map for `.vscode/mcp.json`.
///
/// Combines theta-typed `[tools]` entries with per-server extras from
/// `[harness.github_copilot.tool.<name>]` - theta-typed keys win on conflict.
/// Also emits pure-extras-only servers (in `[harness.github_copilot.tool]`
/// but absent from `[tools]`) so harness-specific servers round-trip.
///
/// ref: <https://code.visualstudio.com/docs/copilot/reference/mcp-configuration>
fn build_mcp_servers(
    manifest: &ThetaManifest,
    cfg: Option<&CopilotConfig>,
) -> JsonMap<String, JsonValue> {
    let mut servers = JsonMap::new();
    let mut sink: Vec<Diagnostic> = Vec::new();

    // theta-typed servers, merged with extras (theta-typed wins)
    //
    // ref: merge semantics live in `merge_json_objects`
    if let Some(ref tools) = manifest.tools {
        for (name, tool) in tools {
            if !tool.enabled {
                continue;
            }
            let mut typed = JsonMap::new();
            if let Some(ref cmd) = tool.command {
                typed.insert("type".into(), JsonValue::String("stdio".into()));
                if let Some(first) = cmd.first() {
                    typed.insert("command".into(), JsonValue::String(first.clone()));
                }
                let extra_cmd_args = &cmd[1..];
                let tool_args = tool.args.as_deref().unwrap_or_default();
                let all_args: Vec<&String> =
                    extra_cmd_args.iter().chain(tool_args.iter()).collect();
                if !all_args.is_empty() {
                    typed.insert(
                        "args".into(),
                        JsonValue::Array(
                            all_args
                                .iter()
                                .map(|a| JsonValue::String((*a).clone()))
                                .collect(),
                        ),
                    );
                }
            }
            if let Some(ref url) = tool.url {
                typed.insert("type".into(), JsonValue::String("http".into()));
                typed.insert("url".into(), JsonValue::String(url.clone()));
            }
            if let Some(ref env) = tool.env {
                let env_obj: JsonMap<String, JsonValue> = env
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
                    .collect();
                typed.insert("env".into(), JsonValue::Object(env_obj));
            }
            if let Some(headers) = tool.headers.as_ref().filter(|h| !h.is_empty()) {
                let hdr_obj: JsonMap<String, JsonValue> = headers
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
                    .collect();
                typed.insert("headers".into(), JsonValue::Object(hdr_obj));
            }

            // merge extras (base) with typed (overlay, wins)
            let extras_map = cfg
                .and_then(|c| c.tool.get(name))
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            let merged = merge_json_objects(
                extras_map,
                typed,
                &format!("{} --> {name}", CopilotLayout::mcp().display()),
                &mut sink,
            );
            servers.insert(name.clone(), JsonValue::Object(merged));
        }
    }

    // pure-extras-only servers (in [harness.github_copilot.tool] but not [tools])
    if let Some(c) = cfg {
        for (name, extras_val) in &c.tool {
            if servers.contains_key(name) {
                continue;
            }
            let extras_obj = extras_val.as_object().cloned().unwrap_or_else(JsonMap::new);
            servers.insert(name.clone(), JsonValue::Object(extras_obj));
        }
    }

    servers
}

/// Build the theta-owned `.vscode/mcp.json` wrapper from `[tools]` +
/// `[harness.github_copilot.tool.*]` + `[harness.github_copilot.mcp_input_variables]`,
/// returning `None` when theta contributes nothing.
///
/// Pure "from-scratch" builder — knows nothing about any existing on-disk file.
fn build_mcp_wrapper(
    manifest: &ThetaManifest,
    cfg: Option<&CopilotConfig>,
) -> Option<JsonMap<String, JsonValue>> {
    let servers = build_mcp_servers(manifest, cfg);
    let inputs = cfg.and_then(|c| c.mcp_input_variables.as_ref());

    if servers.is_empty() && inputs.is_none() {
        return None;
    }

    let mut wrapper = JsonMap::new();
    if !servers.is_empty() {
        wrapper.insert("servers".into(), JsonValue::Object(servers));
    }
    if let Some(v) = inputs {
        wrapper.insert("inputs".into(), v.clone());
    }
    Some(wrapper)
}

/// Merge the theta-owned MCP wrapper over an existing user file.
///
/// Preserves unrelated top-level keys; within `servers`, preserves unrelated
/// server entries (theta-named servers overwrite same-named user entries);
/// within top-level `inputs`, theta's value wins only when present.
fn merge_mcp_wrapper(
    theta: JsonMap<String, JsonValue>,
    existing: &JsonMap<String, JsonValue>,
) -> JsonMap<String, JsonValue> {
    let mut out = existing.clone();

    // merge servers (unrelated preserved, theta overwrites same-named)
    if let Some(JsonValue::Object(theta_servers)) = theta.get("servers") {
        let mut merged_servers = out
            .get("servers")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        for (name, val) in theta_servers {
            merged_servers.insert(name.clone(), val.clone());
        }
        if !merged_servers.is_empty() {
            out.insert("servers".into(), JsonValue::Object(merged_servers));
        }
    }

    // inputs: theta wins when present; otherwise existing `inputs` survive
    if let Some(v) = theta.get("inputs") {
        out.insert("inputs".into(), v.clone());
    }

    out
}

/// Cast `.vscode/mcp.json`: build the theta wrapper, merge with any existing
/// user file, serialize. Early-returns empty when theta contributes nothing,
/// leaving any existing file untouched.
fn cast_mcp_file(
    manifest: &ThetaManifest,
    cfg: Option<&CopilotConfig>,
    existing: Option<&JsonMap<String, JsonValue>>,
) -> Result<Vec<CastFile>> {
    let Some(theta_wrapper) = build_mcp_wrapper(manifest, cfg) else {
        return Ok(Vec::new());
    };

    let merged = match existing {
        Some(existing_map) => merge_mcp_wrapper(theta_wrapper, existing_map),
        None => theta_wrapper,
    };

    let json = serde_json::to_string_pretty(&JsonValue::Object(merged)).map_err(|e| {
        anyhow::anyhow!(
            "failed to serialize {}: {e}",
            CopilotLayout::mcp().display()
        )
    })?;
    Ok(vec![(CopilotLayout::mcp(), json.into())])
}

/// Build `.github/agents/<name>.agent.md` files from `[[subagents]]` and
/// `[harness.github_copilot.subagent.<name>]` extras - theta-typed keys win
///
/// ref: <https://code.visualstudio.com/docs/copilot/customization/custom-agents>
fn cast_subagent_files(
    manifest: &ThetaManifest,
    theta_dir: &Path,
    cfg: Option<&CopilotConfig>,
) -> Result<Vec<CastFile>> {
    let mut out = Vec::new();
    let Some(ref subagents) = manifest.subagents else {
        return Ok(out);
    };

    for subagent in subagents {
        let resolved = crate::common::ResolvedSubagent::load(subagent, theta_dir);
        let description = resolved.description();
        let model = resolved.model();

        let mut fm: Vec<(String, serde_norway::Value)> = Vec::new();
        // harness-specific extras first, so theta-typed keys (pushed below)
        // appear in a predictable order at the end of the frontmatter.
        // extras are keyed by filename stem (kebab slug), not display name
        let slug = theta_static::kebab_case(resolved.name());
        if let Some(extras) = cfg
            .and_then(|c| {
                c.subagent
                    .get(&slug)
                    .or_else(|| c.subagent.get(resolved.name()))
            })
            .and_then(|v| v.as_object())
        {
            for (k, v) in extras {
                if theta_static::THETA_TYPED_AGENT_KEYS.contains(&k.as_str()) {
                    // `name` is implicit via filename / `[[subagents]].name`;
                    // the other typed keys (description, model, tools) are
                    // pushed below from theta-typed fields and win on conflict
                    continue;
                }
                match serde_norway::to_value(v) {
                    Ok(yaml_v) => fm.push((k.clone(), yaml_v)),
                    Err(e) => {
                        tracing::warn!(key = k.as_str(), error = %e, "skipping subagent extra: JSON→YAML conversion failed");
                    }
                }
            }
        }

        let has_model = model.is_some();
        let has_tools = resolved.tools().is_some_and(|t| !t.is_empty());
        let is_default_desc = description == "imported from .github/agents/"
            || description == "imported from copilot";
        let has_extras = !fm.is_empty();

        // only emit frontmatter if there's something meaningful - agents
        // that originally had no frontmatter should stay frontmatter-free
        if has_extras || has_model || has_tools || !is_default_desc {
            fm.push(("name".into(), fm_str(resolved.name())));
            fm.push(("description".into(), fm_str(description)));
            if let Some(m) = model {
                fm.push(("model".into(), fm_str(m)));
            }
            if let Some(tools) = resolved.tools() {
                if !tools.is_empty() {
                    fm.push(("tools".into(), fm_list(tools)));
                }
            }
        }
        let fm_refs: Vec<(&str, serde_norway::Value)> =
            fm.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        let frontmatter = yaml_frontmatter(&fm_refs)?;
        let body = resolved.body("")?;
        out.push((
            CopilotLayout::agent(&slug),
            format!("{frontmatter}{body}").into(),
        ));
    }

    Ok(out)
}

/// Build `.github/hooks/theta-hooks.json` from `[harness.github_copilot].hooks`
///
/// VS Code discovers hooks by scanning every `*.json` file in `.github/hooks/`,
/// so writing all theta-managed hooks to a single consolidated file keeps the
/// reverse round-trip deterministic. Users may add their own `*.json`
/// siblings, which are union-merged on import under
/// `[harness.github_copilot].hooks`
///
/// On-disk shape wraps the event map in a `"hooks"` key, per VS Code's format:
///   `{ "hooks": { "PreToolUse": [...], "PostToolUse": [...] } }`
///
/// ref: <https://code.visualstudio.com/docs/copilot/customization/hooks>
fn cast_hooks_file(cfg: Option<&CopilotConfig>) -> Result<Vec<CastFile>> {
    let Some(hooks) = cfg.and_then(|c| c.hooks.as_ref()) else {
        return Ok(Vec::new());
    };
    let wrapped = serde_json::json!({ "hooks": hooks });
    let json = serde_json::to_string_pretty(&wrapped).map_err(|e| {
        anyhow::anyhow!(
            "failed to serialize {}: {e}",
            CopilotLayout::hooks_file().display()
        )
    })?;
    Ok(vec![(CopilotLayout::hooks_file(), json.into())])
}

/// Shallow-merge the theta-owned settings over an existing user file: theta
/// keys overwrite same-named keys; unrelated user keys are preserved verbatim.
///
/// ref: <https://code.visualstudio.com/docs/copilot/reference/copilot-settings>
fn merge_vscode_settings(
    theta: JsonMap<String, JsonValue>,
    existing: &JsonMap<String, JsonValue>,
) -> JsonMap<String, JsonValue> {
    let mut out = existing.clone();
    for (k, v) in theta {
        out.insert(k, v);
    }
    out
}

/// Cast `.vscode/settings.json`: build the theta settings, merge with any
/// existing user file, serialize. Early-returns empty when theta contributes
/// nothing, leaving any existing file untouched.
fn cast_settings_file(
    manifest: &ThetaManifest,
    existing: Option<&JsonMap<String, JsonValue>>,
) -> Result<Vec<CastFile>> {
    let Some(settings) = build_vscode_settings(manifest)? else {
        return Ok(Vec::new());
    };

    let merged = match existing {
        Some(existing_map) => merge_vscode_settings(settings, existing_map),
        None => settings,
    };

    let json = serde_json::to_string_pretty(&merged)
        .map_err(|e| anyhow::anyhow!("failed to serialize settings.json: {e}"))?;
    Ok(vec![(CopilotLayout::settings(), json.into())])
}

/// Build the `.vscode/settings.json` payload from `[harness.github_copilot]`
/// config, returning `None` when there's nothing to write.
fn build_vscode_settings(manifest: &ThetaManifest) -> Result<Option<JsonMap<String, JsonValue>>> {
    let key = theta_harness::HarnessTarget::Copilot.toml_key();
    let cfg: Option<CopilotConfig> = manifest
        .harness_config(key)
        .map_err(|e| anyhow::anyhow!("[harness.{key}]: failed to parse: {e}"))?;

    let Some(cfg) = cfg else {
        return Ok(None);
    };

    let mut settings = JsonMap::new();

    for (key, value) in &cfg.extra {
        settings.insert(key.clone(), value.clone());
    }

    if settings.is_empty() {
        return Ok(None);
    }

    Ok(Some(settings))
}

pub(super) fn validate_config(manifest: &ThetaManifest) -> Vec<Diagnostic> {
    let key = theta_harness::HarnessTarget::Copilot.toml_key();
    let cfg = match manifest.harness_config::<CopilotConfig>(key) {
        Ok(Some(cfg)) => cfg,
        Ok(None) => {
            let mut diags = crate::common::collect_lossy_apply_warnings(manifest, "GitHub Copilot");
            diags.extend(crate::common::collect_env_placeholder_warnings(
                manifest,
                theta_harness::HarnessTarget::Copilot,
            ));
            return diags;
        }
        Err(e) => {
            return vec![Diagnostic::warn(
                format!("[harness.{key}]"),
                format!("failed to parse {key} config: {e}"),
            )];
        }
    };
    let mut diags = crate::harness_config::validate_version(&cfg);
    diags.extend(crate::common::collect_lossy_apply_warnings(
        manifest,
        "GitHub Copilot",
    ));
    diags.extend(crate::common::collect_env_placeholder_warnings(
        manifest,
        theta_harness::HarnessTarget::Copilot,
    ));

    // subagent extras vs theta-typed [[subagents]] conflict detection
    //   theta-typed wins; warn when [harness.github_copilot.subagent.<name>] shadows a key
    //   already set by the theta-typed subagent entry
    if !cfg.subagent.is_empty() {
        if let Some(ref subagents) = manifest.subagents {
            for sa in subagents {
                let Some(extras) = cfg.subagent.get(&sa.name).and_then(|v| v.as_object()) else {
                    continue;
                };
                let mut typed_keys: Vec<&'static str> = vec!["description"];
                if sa.model.is_some() {
                    typed_keys.push("model");
                }
                if sa.tools.as_ref().is_some_and(|t| !t.is_empty()) {
                    typed_keys.push("tools");
                }
                for k in typed_keys {
                    if extras.contains_key(k) {
                        diags.push(Diagnostic::warn(
                            format!("[harness.github_copilot.subagent.{}]", sa.name),
                            format!(
                                "key `{k}` is also set by theta-typed [[subagents]]; the theta-typed value wins - remove it from the harness extras or update [[subagents]]"
                            ),
                        ));
                    }
                }
                // `name` in extras is always redundant (implicit via filename / [[subagents]].name)
                if extras.contains_key("name") {
                    diags.push(Diagnostic::hint(
                        format!("[harness.github_copilot.subagent.{}]", sa.name),
                        "`name` is redundant - the name is derived from the table key / `[[subagents]].name`",
                    ));
                }
            }
        }
    }

    // MCP extras vs theta-typed [tools] conflict detection
    //   theta-typed wins; warn when [harness.github_copilot.tool.<name>] shadows a key
    //   that would be set by the theta-typed tool entry
    if !cfg.tool.is_empty() {
        if let Some(ref tools) = manifest.tools {
            for (name, tool) in tools {
                if !tool.enabled {
                    continue;
                }
                let Some(extras) = cfg.tool.get(name).and_then(|v| v.as_object()) else {
                    continue;
                };
                // reconstruct the theta-typed keys that will be set for this server
                let mut typed_keys: Vec<&'static str> = Vec::new();
                if tool.command.is_some() {
                    typed_keys.push("type");
                    typed_keys.push("command");
                    if tool.command.as_ref().map_or(0, std::vec::Vec::len) > 1
                        || tool.args.as_ref().is_some_and(|a| !a.is_empty())
                    {
                        typed_keys.push("args");
                    }
                }
                if tool.url.is_some() {
                    typed_keys.push("type");
                    typed_keys.push("url");
                }
                if tool.env.is_some() {
                    typed_keys.push("env");
                }
                if tool.headers.as_ref().is_some_and(|h| !h.is_empty()) {
                    typed_keys.push("headers");
                }
                for k in typed_keys {
                    if extras.contains_key(k) {
                        diags.push(Diagnostic::warn(
                            format!("[harness.github_copilot.tool.{name}]"),
                            format!(
                                "key `{k}` is also set by theta-typed [tools.{name}]; the theta-typed value wins - remove it from the harness extras or update [tools.{name}]"
                            ),
                        ));
                    }
                }
            }
        }
    }

    diags
}
