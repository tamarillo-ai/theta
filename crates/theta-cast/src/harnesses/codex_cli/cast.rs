//! Codex CLI caster: theta manifest --> codex-native files.
//!
//! See `mod.rs` for the surface inventory and ref URLs.

use std::path::{Path, PathBuf};

use crate::Caster;
use crate::common::{
    CastContent, CastFile, ResolvedSubagent, build_system_prompt, json_to_toml_item,
    merge_json_objects, read_all_rules, read_skill_dir_files,
};
use crate::harness_config::CodexCliConfig;
use anyhow::Result;
use theta_harness::layout::{CodexCliLayout, HarnessLayout};
use theta_schema::{Diagnostic, ThetaManifest};

use super::CodexCli;

/// Default `project_doc_max_bytes` is 32 KiB. Files past the cap are silently
/// dropped by codex (NOT truncated mid-document).
/// ref: <https://developers.openai.com/codex/guides/agents-md>
const PROJECT_DOC_MAX_BYTES: usize = 32 * 1024;

// Codex on-disk key names live on `CodexCliLayout` as associated constants:
// `SPECIFIC_SKILLS_KEY`, `HOOKS_KEY`, `MCP_ROOT_KEY`, `MCP_HTTP_HEADERS_KEY`,
// `AGENT_KEY_NAME` / `AGENT_KEY_DESCRIPTION` / `AGENT_KEY_DEV_INSTRUCTIONS`
// / `AGENT_KEY_MODEL`, `TYPED_MCP_KEYS`, `TYPED_AGENT_KEYS`.

impl Caster for CodexCli {
    fn cast_files(&self, manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
        let mut files = Vec::new();

        let key = theta_harness::HarnessTarget::CodexCli.toml_key();
        let cfg: Option<CodexCliConfig> = manifest
            .harness_config(key)
            .map_err(|e| anyhow::anyhow!("[harness.{key}]: failed to parse: {e}"))?;

        // system prompt + rules --> AGENTS.md (opaque body, monolithic).
        // codex AGENTS.md is treated as opaque developer-instructions content;
        // theta does NOT prepend `# name` / description (decision D4: identity
        // comes from project dir, never from AGENTS.md body).
        // codex has no per-rule conditional activation, so all rules are
        // appended as `## <name>` H2 sections joined by `---`. import has no
        // reverse-split (decision D6: WONTFIX lossy by design).
        // ref: <https://developers.openai.com/codex/guides/agents-md>
        let mut sections = Vec::new();
        if let Some(body) = build_system_prompt(manifest, theta_dir)? {
            sections.push(body);
        }
        for (name, _rule, content) in read_all_rules(manifest, theta_dir)? {
            sections.push(format!("## {name}\n\n{content}"));
        }
        if !sections.is_empty() {
            files.push((
                CodexCliLayout::agents_md(),
                sections.join("\n\n---\n\n").into(),
            ));
        }

        // .codex/config.toml (top-level keys + [mcp_servers.*])
        // ref: <https://developers.openai.com/codex/config-reference>
        let mut sink: Vec<Diagnostic> = Vec::new();
        if let Some(config_toml) = build_config_toml(manifest, cfg.as_ref(), &mut sink)? {
            files.push((CodexCliLayout::config(), config_toml.into()));
        }

        // .codex/hooks.json from [harness.codex.hooks]. cast always emits as
        // JSON (decision D2: never inline). codex warns when both inline
        // [hooks] and hooks.json exist in the same layer; cast avoids that by
        // construction.
        // ref: <https://developers.openai.com/codex/hooks>
        if let Some(hooks_file) = build_hooks_json(cfg.as_ref())? {
            files.push((CodexCliLayout::hooks(), hooks_file.into()));
        }

        // [[subagents]] + [harness.codex.subagent.<slug>] --> .codex/agents/<slug>.toml
        // each codex subagent file is a full config layer: theta-typed fields
        // are emitted at the top, extras are merged underneath, theta-typed
        // wins on conflict (with a warn diagnostic).
        // ref: <https://developers.openai.com/codex/subagents>
        files.extend(cast_subagent_files(
            manifest,
            theta_dir,
            cfg.as_ref(),
            &mut sink,
        )?);

        // [skills] --> .agents/skills/<name>/SKILL.md (default) or
        // .codex/skills/<name>/SKILL.md (with codex_specific_skills=true).
        // both paths are documented; canonical wins by default.
        // ref: <https://developers.openai.com/codex/skills>
        if let Some(ref skills) = manifest.skills {
            let skills_dir = skill_output_dir(cfg.as_ref());
            for name in skills.keys() {
                let skill_files = read_skill_dir_files(theta_dir, name, &skills_dir)?;
                files.extend(skill_files);
            }
        }

        Ok(files)
    }

    fn validate_output(&self, files: &[CastFile]) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        // codex truncates AGENTS.md once the cumulative size of all discovered
        // AGENTS.md files reaches project_doc_max_bytes; files past the cap
        // are SILENTLY DROPPED. cast emits a single root AGENTS.md, so the
        // check is against this one file's byte length.
        // ref: <https://developers.openai.com/codex/guides/agents-md>
        for (path, content) in files {
            if path != &CodexCliLayout::agents_md() {
                continue;
            }
            let size = match content {
                CastContent::Text(s) => s.len(),
                CastContent::Binary(b) => b.len(),
            };
            if size > PROJECT_DOC_MAX_BYTES {
                diags.push(Diagnostic::warn(
                    path.display().to_string(),
                    format!(
                        "AGENTS.md is {size} bytes; codex truncates the cumulative project doc set at project_doc_max_bytes (default {PROJECT_DOC_MAX_BYTES} bytes). content past the cap is silently dropped."
                    ),
                ));
            }
        }
        diags
    }

    fn validate_config(&self, manifest: &ThetaManifest) -> Vec<Diagnostic> {
        let key = theta_harness::HarnessTarget::CodexCli.toml_key();
        let cfg = match manifest.harness_config::<CodexCliConfig>(key) {
            Ok(Some(cfg)) => cfg,
            Ok(None) => {
                let mut diags = crate::common::collect_lossy_apply_warnings(manifest, "Codex CLI");
                diags.extend(crate::common::collect_env_placeholder_warnings(
                    manifest,
                    theta_harness::HarnessTarget::CodexCli,
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
            "Codex CLI",
        ));
        diags.extend(crate::common::collect_env_placeholder_warnings(
            manifest,
            theta_harness::HarnessTarget::CodexCli,
        ));
        diags
    }
}

/// Choose the skills output directory based on the `codex_specific_skills`
/// knob under `[harness.codex]`. Defaults to canonical `.agents/skills/`.
fn skill_output_dir(cfg: Option<&CodexCliConfig>) -> PathBuf {
    let opt_in = cfg
        .and_then(|c| c.extra.get(CodexCliLayout::SPECIFIC_SKILLS_KEY))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if opt_in {
        CodexCliLayout::codex_skills_dir()
    } else {
        CodexCliLayout::cross_agent_skills_dir()
    }
}

// .codex/config.toml

/// Build `.codex/config.toml` by:
/// - emitting `[mcp_servers.<name>]` tables from `[tools]` merged with
///   `[harness.codex.tool.<name>]` extras (theta-typed wins on conflict),
/// - emitting every top-level key from `[harness.codex]` (minus theta-internal
///   knobs and the `hooks` table, which goes to hooks.json).
///
/// returns None if the document would be empty.
/// ref: <https://developers.openai.com/codex/config-reference>
fn build_config_toml(
    manifest: &ThetaManifest,
    cfg: Option<&CodexCliConfig>,
    diags: &mut Vec<Diagnostic>,
) -> Result<Option<String>> {
    let mut doc = toml_edit::DocumentMut::new();

    // [tools] + per-server extras --> [mcp_servers.<name>]
    let mcp_servers = build_mcp_servers(manifest, cfg, diags);
    if !mcp_servers.is_empty() {
        let mcp_table = doc
            .as_table_mut()
            .entry(CodexCliLayout::MCP_ROOT_KEY)
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()));
        let Some(mcp_table) = mcp_table.as_table_mut() else {
            return Err(anyhow::anyhow!(
                "internal invariant violated: [{}] must be a table",
                CodexCliLayout::MCP_ROOT_KEY
            ));
        };
        for (name, server_json) in mcp_servers {
            let item = json_to_toml_item(&serde_json::Value::Object(server_json));
            if let Ok(table) = item.into_table() {
                mcp_table[name.as_str()] = toml_edit::Item::Table(table);
            }
        }
    }

    // [harness.codex] top-level keys --> config.toml top-level
    // skip the theta-internal knob and the hooks table (which routes to hooks.json)
    if let Some(cfg) = cfg {
        for (key, value) in &cfg.extra {
            if key == CodexCliLayout::SPECIFIC_SKILLS_KEY || key == CodexCliLayout::HOOKS_KEY {
                continue;
            }
            let item = json_to_toml_item(value);
            if !item.is_none() {
                doc[key.as_str()] = item;
            }
        }
    }

    let output = doc.to_string();
    if output.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(output))
}

/// Build the `[mcp_servers.*]` map by combining theta-typed `[tools]` with
/// per-server extras from `[harness.codex.tool.<name>]`. theta-typed fields
/// win on conflict.
fn build_mcp_servers(
    manifest: &ThetaManifest,
    cfg: Option<&CodexCliConfig>,
    diags: &mut Vec<Diagnostic>,
) -> std::collections::BTreeMap<String, serde_json::Map<String, serde_json::Value>> {
    use serde_json::{Map as JsonMap, Value as JsonValue};
    let mut servers: std::collections::BTreeMap<String, JsonMap<String, JsonValue>> =
        std::collections::BTreeMap::new();

    if let Some(ref tools) = manifest.tools {
        for (name, tool) in tools {
            // Codex CLI uniquely supports `enabled = false` to register a
            // server as inactive (per `developers.openai.com/codex/mcp`).
            let mut typed = JsonMap::new();
            if !tool.enabled {
                typed.insert("enabled".into(), JsonValue::Bool(false));
            }
            if let Some(ref cmd) = tool.command {
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
                    CodexCliLayout::MCP_HTTP_HEADERS_KEY.into(),
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
                &format!(
                    "{} --> {}.{name}",
                    CodexCliLayout::config().display(),
                    CodexCliLayout::MCP_ROOT_KEY,
                ),
                diags,
            );
            servers.insert(name.clone(), merged);
        }
    }

    // pure-extras-only servers (declared in [harness.codex.tool.*] but absent
    // from [tools]) round-trip too.
    if let Some(c) = cfg {
        for (name, extras_val) in &c.tool {
            if servers.contains_key(name) {
                continue;
            }
            let extras_obj = extras_val.as_object().cloned().unwrap_or_default();
            servers.insert(name.clone(), extras_obj);
        }
    }

    servers
}

// .codex/hooks.json

/// Build `.codex/hooks.json` from `[harness.codex.hooks]`. Returns None when
/// there are no hooks to write. cast always uses the JSON file form (never
/// inline `[hooks]` in config.toml) per decision D2.
/// ref: <https://developers.openai.com/codex/hooks>
fn build_hooks_json(cfg: Option<&CodexCliConfig>) -> Result<Option<String>> {
    let hooks_value = cfg.and_then(|c| c.extra.get(CodexCliLayout::HOOKS_KEY));
    let Some(hooks_value) = hooks_value else {
        return Ok(None);
    };
    // accept any JSON shape codex would accept; theta does not validate event
    // names (the codex schema folder ships events the docs page does not
    // list - we treat the enum as open).
    if hooks_value.is_null() {
        return Ok(None);
    }
    let json = serde_json::to_string_pretty(hooks_value).map_err(|e| {
        anyhow::anyhow!(
            "failed to serialize {}: {e}",
            CodexCliLayout::hooks().display()
        )
    })?;
    Ok(Some(json))
}

// .codex/agents/<slug>.toml

/// Build one `.codex/agents/<slug>.toml` per `[[subagents]]` entry. each file
/// is a full config layer: theta-typed fields go at the top, extras (a full
/// `config.toml` subtree) merge underneath, theta-typed wins on conflict.
/// ref: <https://developers.openai.com/codex/subagents>
fn cast_subagent_files(
    manifest: &ThetaManifest,
    theta_dir: &Path,
    cfg: Option<&CodexCliConfig>,
    diags: &mut Vec<Diagnostic>,
) -> Result<Vec<CastFile>> {
    use serde_json::{Map as JsonMap, Value as JsonValue};
    let mut out = Vec::new();
    let Some(ref subagents) = manifest.subagents else {
        return Ok(out);
    };

    for subagent in subagents {
        let resolved = ResolvedSubagent::load(subagent, theta_dir);
        let slug = theta_static::kebab_case(resolved.name());

        // typed fields from the subagent declaration.
        let mut typed = JsonMap::new();
        typed.insert(
            CodexCliLayout::AGENT_KEY_NAME.into(),
            JsonValue::String(resolved.name().into()),
        );

        let description = resolved.description();
        if !description.is_empty() {
            typed.insert(
                CodexCliLayout::AGENT_KEY_DESCRIPTION.into(),
                JsonValue::String(description.into()),
            );
        }

        let body = resolved.body("")?;
        if !body.trim().is_empty() {
            typed.insert(
                CodexCliLayout::AGENT_KEY_DEV_INSTRUCTIONS.into(),
                JsonValue::String(body),
            );
        }

        if let Some(m) = resolved.model() {
            typed.insert(
                CodexCliLayout::AGENT_KEY_MODEL.into(),
                JsonValue::String(m.into()),
            );
        }

        // extras keyed by slug; if author wrote the manifest with the display
        // name instead, also try the un-kebabed name.
        let extras_map = cfg
            .and_then(|c| {
                c.subagent
                    .get(&slug)
                    .or_else(|| c.subagent.get(resolved.name()))
            })
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let merged = merge_json_objects(
            extras_map,
            typed,
            &CodexCliLayout::agent(&slug).display().to_string(),
            diags,
        );

        let item = json_to_toml_item(&JsonValue::Object(merged));
        let table = item.into_table().map_err(|_| {
            anyhow::anyhow!("internal invariant violated: subagent file must serialize as a table")
        })?;
        let mut doc = toml_edit::DocumentMut::new();
        for (k, v) in &table {
            doc[k] = v.clone();
        }

        out.push((CodexCliLayout::agent(&slug), doc.to_string().into()));
    }

    Ok(out)
}
