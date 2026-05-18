//! Claude Code caster: theta manifest —> claude-native files.
//!
//! See `mod.rs` for the surface inventory and ref URLs.

use std::path::Path;

use crate::Caster;
use crate::common::{
    CastContent, CastFile, ResolvedSubagent, build_system_prompt, fm_list, fm_str,
    merge_json_objects, read_all_rules, read_skill_dir_files, yaml_frontmatter,
};
use crate::harness_config::ClaudeCodeConfig;
use anyhow::Result;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use theta_harness::layout::{ClaudeCodeLayout, HarnessLayout};
use theta_schema::{ApplyMode, Diagnostic, ThetaManifest};

use super::ClaudeCode;

/// Soft size limit from Claude's guidance.
/// ref: <https://code.claude.com/docs/en/memory#write-effective-instructions>
const RECOMMENDED_MAX_LINES: usize = 200;

impl Caster for ClaudeCode {
    fn cast_files(&self, manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
        let mut files = Vec::new();

        let key = theta_harness::HarnessTarget::ClaudeCode.toml_key();
        let cfg: Option<ClaudeCodeConfig> = manifest
            .harness_config(key)
            .map_err(|e| anyhow::anyhow!("[harness.{key}]: failed to parse: {e}"))?;

        // system prompt to `CLAUDE.md` when `[instructions].system` is set.
        // claude reads `CLAUDE.md` verbatim; agent name/description live in
        // `[agent]` and are not re-emitted as a synthetic header. cast always
        // writes the canonical project-root location, even if import read the
        // alternate `.claude/CLAUDE.md` path.
        // ref: <https://code.claude.com/docs/en/memory#choose-where-to-put-claude-md-files>
        if let Some(content) = build_system_prompt(manifest, theta_dir)? {
            files.push((ClaudeCodeLayout::system_prompt(), content.into()));
        }

        // per-rule files. path-qualified names (`frontend/api-design`)
        // preserve the source subdir layout; claude discovers rules
        // recursively so subdirectories are first-class.
        // ref: <https://code.claude.com/docs/en/memory#organize-rules-with-claude-rules>
        for (name, rule, content) in read_all_rules(manifest, theta_dir)? {
            let mut fm_entries: Vec<(&str, serde_norway::Value)> = Vec::new();
            if rule.apply == ApplyMode::Glob {
                if let Some(ref patterns) = rule.apply_to {
                    fm_entries.push(("paths", fm_list(patterns)));
                }
            }
            let frontmatter = yaml_frontmatter(&fm_entries)?;
            files.push((
                ClaudeCodeLayout::rule(&name),
                format!("{frontmatter}{content}").into(),
            ));
        }

        // harness settings to `.claude/settings.json`. every key under
        // `[harness.claude_code]` emits with its native camelCase name.
        // ref: <https://code.claude.com/docs/en/settings>
        if let Some(settings) = build_settings_json(cfg.as_ref()) {
            let json = serde_json::to_string_pretty(&settings).map_err(|e| {
                anyhow::anyhow!(
                    "failed to serialize {}: {e}",
                    ClaudeCodeLayout::settings().display()
                )
            })?;
            files.push((ClaudeCodeLayout::settings(), json.into()));
        }

        // `[skills]` to `.claude/skills/<name>/` (full directory).
        // ref: <https://code.claude.com/docs/en/skills>
        if let Some(ref skills) = manifest.skills {
            for name in skills.keys() {
                let skill_files =
                    read_skill_dir_files(theta_dir, name, &ClaudeCodeLayout::skills_dir())?;
                files.extend(skill_files);
            }
        }

        // `[tools]` and `[harness.claude_code.tool.<name>]` to `.mcp.json`.
        // theta-typed keys win over per-server extras on conflict; pure
        // extras-only servers round-trip too.
        // ref: <https://code.claude.com/docs/en/mcp#project-scope>
        files.extend(cast_mcp_file(manifest, cfg.as_ref())?);

        // `[[subagents]]` and `[harness.claude_code.subagent.<slug>]` to
        // `.claude/agents/<name>.md`.
        // ref: <https://code.claude.com/docs/en/sub-agents#write-subagent-files>
        files.extend(cast_subagent_files(manifest, theta_dir, cfg.as_ref())?);

        Ok(files)
    }

    fn validate_output(&self, files: &[CastFile]) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        for (path, content) in files {
            let CastContent::Text(s) = content else {
                continue;
            };
            let lines = s.lines().count();
            if lines > RECOMMENDED_MAX_LINES {
                diags.push(Diagnostic::hint(
                    path.display().to_string(),
                    format!(
                        "{lines} lines — claude recommends <{RECOMMENDED_MAX_LINES} per file for better adherence"
                    ),
                ));
            }
        }
        diags
    }

    fn validate_config(&self, manifest: &ThetaManifest) -> Vec<Diagnostic> {
        validate_config(manifest)
    }
}

// settings.json

/// Build `.claude/settings.json` from `[harness.claude_code]`. Every key on
/// `cfg.extra` passes through verbatim with its original camelCase name —
/// Claude-spec keys (sandbox, hooks, permissions, enabledPlugins, autoMode,
/// viewMode, attribution, worktree, env, statusLine, ...) all live there.
///
/// `tool` and `subagent` are explicitly NOT `settings.json` content — they live
/// under `[harness.claude_code.*]` in `theta.toml` but route to `.mcp.json` and
/// `.claude/agents/` on cast, so they don't appear here.
fn build_settings_json(cfg: Option<&ClaudeCodeConfig>) -> Option<JsonMap<String, JsonValue>> {
    let cc = cfg?;
    if cc.extra.is_empty() {
        return None;
    }
    let mut settings = JsonMap::new();
    for (k, v) in &cc.extra {
        settings.insert(k.clone(), v.clone());
    }
    Some(settings)
}

// .mcp.json

/// Build the `.mcp.json` `mcpServers` map by combining theta-typed `[tools]` with
/// per-server extras from `[harness.claude_code.tool.<name>]`. theta-typed
/// fields win on conflict (via `merge_json_objects`). Pure-extras-only servers
/// (in extras but absent from `[tools]`) round-trip too.
fn build_mcp_servers(
    manifest: &ThetaManifest,
    cfg: Option<&ClaudeCodeConfig>,
) -> JsonMap<String, JsonValue> {
    let mut servers = JsonMap::new();
    let mut sink: Vec<Diagnostic> = Vec::new();

    if let Some(ref tools) = manifest.tools {
        for (name, tool) in tools {
            if !tool.enabled {
                continue;
            }
            let mut typed = JsonMap::new();
            if let Some(ref cmd) = tool.command {
                typed.insert(
                    theta_static::MCP_KEY_TYPE.into(),
                    JsonValue::String(theta_static::McpTransport::Stdio.as_str().into()),
                );
                if let Some(first) = cmd.first() {
                    typed.insert(
                        theta_static::MCP_KEY_COMMAND.into(),
                        JsonValue::String(first.clone()),
                    );
                }
                let extra_cmd_args = &cmd[1..];
                let tool_args = tool.args.as_deref().unwrap_or_default();
                let all_args: Vec<&String> =
                    extra_cmd_args.iter().chain(tool_args.iter()).collect();
                if !all_args.is_empty() {
                    typed.insert(
                        theta_static::MCP_KEY_ARGS.into(),
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
                typed.insert(
                    theta_static::MCP_KEY_TYPE.into(),
                    JsonValue::String(theta_static::McpTransport::Http.as_str().into()),
                );
                typed.insert(
                    theta_static::MCP_KEY_URL.into(),
                    JsonValue::String(url.clone()),
                );
            }
            if let Some(ref env) = tool.env {
                let env_obj: JsonMap<String, JsonValue> = env
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
                    .collect();
                typed.insert(theta_static::MCP_KEY_ENV.into(), JsonValue::Object(env_obj));
            }
            if let Some(headers) = tool.headers.as_ref().filter(|h| !h.is_empty()) {
                let hdr_obj: JsonMap<String, JsonValue> = headers
                    .iter()
                    .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
                    .collect();
                typed.insert(
                    theta_static::MCP_KEY_HEADERS.into(),
                    JsonValue::Object(hdr_obj),
                );
            }

            let extras_map = cfg
                .and_then(|c| c.tool.get(name))
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            let merged = merge_json_objects(
                extras_map,
                typed,
                &format!("{} --> {name}", ClaudeCodeLayout::mcp().display()),
                &mut sink,
            );
            servers.insert(name.clone(), JsonValue::Object(merged));
        }
    }

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

fn cast_mcp_file(
    manifest: &ThetaManifest,
    cfg: Option<&ClaudeCodeConfig>,
) -> Result<Vec<CastFile>> {
    let servers = build_mcp_servers(manifest, cfg);
    if servers.is_empty() {
        return Ok(Vec::new());
    }
    let mut wrapper = JsonMap::new();
    wrapper.insert(
        ClaudeCodeLayout::MCP_ROOT_KEY.into(),
        JsonValue::Object(servers),
    );
    let json = serde_json::to_string_pretty(&JsonValue::Object(wrapper)).map_err(|e| {
        anyhow::anyhow!(
            "failed to serialize {}: {e}",
            ClaudeCodeLayout::mcp().display()
        )
    })?;
    Ok(vec![(ClaudeCodeLayout::mcp(), json.into())])
}

// subagents

/// Build `.claude/agents/<name>.md` files from `[[subagents]]` and
/// `[harness.claude_code.subagent.<slug>]` extras. theta-typed keys win.
/// Agents that originally had no frontmatter and no theta-typed fields stay
/// frontmatter-free to avoid round-trip drift.
fn cast_subagent_files(
    manifest: &ThetaManifest,
    theta_dir: &Path,
    cfg: Option<&ClaudeCodeConfig>,
) -> Result<Vec<CastFile>> {
    let mut out = Vec::new();
    let Some(ref subagents) = manifest.subagents else {
        return Ok(out);
    };

    for subagent in subagents {
        let resolved = ResolvedSubagent::load(subagent, theta_dir);
        let description = resolved.description();
        let model = resolved.model();

        let mut fm: Vec<(String, serde_norway::Value)> = Vec::new();
        // extras first, so theta-typed keys (pushed below) appear in a
        // predictable order. extras keyed by filename stem (kebab slug).
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
                    // theta-typed keys are pushed below from typed fields and
                    // win on conflict. `name` is implicit via filename.
                    continue;
                }
                match serde_norway::to_value(v) {
                    Ok(yaml_v) => fm.push((k.clone(), yaml_v)),
                    Err(e) => {
                        tracing::warn!(
                            key = k.as_str(),
                            error = %e,
                            "skipping subagent extra: JSON→YAML conversion failed"
                        );
                    }
                }
            }
        }

        let has_model = model.is_some();
        let has_tools = resolved.tools().is_some_and(|t| !t.is_empty());
        let has_skills = resolved.skills().is_some_and(|s| !s.is_empty());
        let has_extras = !fm.is_empty();
        let has_description = !description.is_empty();

        // emit frontmatter only when there's something to emit. agents that
        // originally had no frontmatter (import does not fabricate `description`
        // on missing frontmatter, see `import_one_agent_file`) stay frontmatter free.
        if has_extras || has_model || has_tools || has_skills || has_description {
            // theta-typed slots overwrite any colliding extras key, matching
            // `merge_json_objects` ordering for MCP.
            fm.retain(|(k, _)| {
                !matches!(
                    k.as_str(),
                    "name" | "description" | "model" | "tools" | "skills"
                )
            });

            // claude marks `name` as required in subagent frontmatter, and the
            // filename-fallback is a runtime convenience rather than a recommended
            // form. always emit it so the round-trip output matches the documented
            // shape; agents that originally had no `name:` line gain one, which is
            // a documented (and intentional) round-trip normalization.
            // ref: <https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields>
            fm.push(("name".into(), fm_str(resolved.name())));
            if has_description {
                fm.push(("description".into(), fm_str(description)));
            }
            if let Some(m) = model {
                fm.push(("model".into(), fm_str(m)));
            }
            if let Some(tools) = resolved.tools() {
                if !tools.is_empty() {
                    // claude's canonical inline form is a comma-separated
                    // string. emitting a yaml block sequence is functionally
                    // equivalent but causes round-trip drift on every agent.
                    fm.push(("tools".into(), fm_str(tools.join(", "))));
                }
            }
            if let Some(skills) = resolved.skills() {
                if !skills.is_empty() {
                    fm.push(("skills".into(), fm_list(skills)));
                }
            }
        }

        let fm_refs: Vec<(&str, serde_norway::Value)> =
            fm.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        let frontmatter = yaml_frontmatter(&fm_refs)?;
        let body = resolved.body("")?;
        out.push((
            ClaudeCodeLayout::agent(&slug),
            format!("{frontmatter}{body}").into(),
        ));
    }

    Ok(out)
}
// validation

/// Validate `[harness.claude_code]` against the theta-typed manifest sections,
/// emitting warnings when an extras key would be shadowed by a theta-typed
/// field on cast. Mirrors the conflict detection used by copilot.
fn validate_config(manifest: &ThetaManifest) -> Vec<Diagnostic> {
    let key = theta_harness::HarnessTarget::ClaudeCode.toml_key();
    let cc = match manifest.harness_config::<ClaudeCodeConfig>(key) {
        Ok(Some(cc)) => cc,
        Ok(None) => return Vec::new(),
        Err(e) => {
            return vec![Diagnostic::warn(
                format!("[harness.{key}]"),
                format!("failed to parse {key} config: {e}"),
            )];
        }
    };
    let mut diags = crate::harness_config::validate_version(&cc);

    validate_subagent_extras(&cc, manifest, key, &mut diags);
    validate_tool_extras(&cc, manifest, key, &mut diags);
    diags.extend(crate::common::collect_lossy_apply_warnings(
        manifest,
        "Claude Code",
    ));
    diags.extend(crate::common::collect_env_placeholder_warnings(
        manifest,
        theta_harness::HarnessTarget::ClaudeCode,
    ));

    diags
}

/// `[harness.claude_code.subagent.<slug>]` vs theta-typed `[[subagents]]`:
/// theta-typed wins on cast. Emit a warning per shadowed key.
fn validate_subagent_extras(
    cc: &ClaudeCodeConfig,
    manifest: &ThetaManifest,
    key: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if cc.subagent.is_empty() {
        return;
    }
    let Some(ref subagents) = manifest.subagents else {
        return;
    };

    for sa in subagents {
        let slug = theta_static::kebab_case(&sa.name);
        let Some(extras) = cc
            .subagent
            .get(&slug)
            .or_else(|| cc.subagent.get(&sa.name))
            .and_then(|v| v.as_object())
        else {
            continue;
        };

        let mut typed_keys: Vec<&'static str> = vec!["description"];
        if sa.model.is_some() {
            typed_keys.push("model");
        }
        if sa.tools.as_ref().is_some_and(|t| !t.is_empty()) {
            typed_keys.push("tools");
        }
        if sa.skills.as_ref().is_some_and(|s| !s.is_empty()) {
            typed_keys.push("skills");
        }
        for k in typed_keys {
            if extras.contains_key(k) {
                diags.push(Diagnostic::warn(
                    format!("[harness.{key}.subagent.{slug}]"),
                    format!(
                        "key `{k}` is also set by theta-typed [[subagents]]; the theta-typed value wins — remove it from the harness extras or update [[subagents]]"
                    ),
                ));
            }
        }
        if extras.contains_key("name") {
            diags.push(Diagnostic::hint(
                format!("[harness.{key}.subagent.{slug}]"),
                "`name` is redundant — the name is derived from the filename / `[[subagents]].name`",
            ));
        }
    }
}

/// `[harness.claude_code.tool.<name>]` vs theta-typed `[tools]`: theta-typed
/// wins on cast. Emit a warning per shadowed key.
fn validate_tool_extras(
    cc: &ClaudeCodeConfig,
    manifest: &ThetaManifest,
    key: &str,
    diags: &mut Vec<Diagnostic>,
) {
    if cc.tool.is_empty() {
        return;
    }
    let Some(ref tools) = manifest.tools else {
        return;
    };

    for (name, tool) in tools {
        if !tool.enabled {
            continue;
        }
        let Some(extras) = cc.tool.get(name).and_then(|v| v.as_object()) else {
            continue;
        };
        let mut typed_keys: Vec<&'static str> = Vec::new();
        if tool.command.is_some() {
            typed_keys.push("command");
            typed_keys.push("type");
            typed_keys.push("args");
        }
        if tool.url.is_some() {
            typed_keys.push("url");
            typed_keys.push("type");
        }
        if tool.env.is_some() {
            typed_keys.push("env");
        }
        if tool.headers.is_some() {
            typed_keys.push("headers");
        }
        for k in typed_keys {
            if extras.contains_key(k) {
                diags.push(Diagnostic::warn(
                    format!("[harness.{key}.tool.{name}]"),
                    format!(
                        "key `{k}` is also set by theta-typed [tools.{name}]; the theta-typed value wins — remove it from the harness extras or update [tools]"
                    ),
                ));
            }
        }
    }
}
