use std::path::Path;

use crate::common::{
    CastFile, ResolvedSubagent, fm_bool, fm_str, identity_and_system, read_all_rules,
    read_skill_dir_files, yaml_frontmatter,
};
use crate::harness_config::CursorConfig;
use anyhow::Result;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use theta_harness::layout::{CursorLayout, HarnessLayout};
use theta_schema::{ApplyMode, ThetaManifest};

/// Emit `.mdc` frontmatter as raw `key: value` lines (NOT YAML).
///
/// Cursor's frontmatter is a line-based format, not YAML. Using `serde_norway`
/// would quote values with `*` or `:`, which Cursor reads literally.
fn mdc_frontmatter(entries: &[(&str, &str)]) -> String {
    if entries.is_empty() {
        return String::new();
    }
    let mut out = String::from("---\n");
    for (k, v) in entries {
        out.push_str(k);
        out.push_str(": ");
        out.push_str(v);
        out.push('\n');
    }
    out.push_str("---\n");
    out
}

// cast: system prompt
// ref: https://cursor.com/docs/rules

fn cast_system_prompt(manifest: &ThetaManifest, theta_dir: &Path) -> Result<CastFile> {
    let system_sections = identity_and_system(manifest, theta_dir)?;
    let fm = mdc_frontmatter(&[
        (super::MDC_DESCRIPTION, "System prompt and agent identity"),
        (super::MDC_ALWAYS_APPLY, "true"),
    ]);
    Ok((
        CursorLayout::system_prompt(),
        format!("{}{}", fm, system_sections.join("\n\n")).into(),
    ))
}

// cast: rules
// ref: https://cursor.com/docs/rules

fn cast_rules(manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
    let mut files = Vec::new();

    for (name, rule, content) in read_all_rules(manifest, theta_dir)? {
        let mut entries: Vec<(&str, String)> = Vec::new();

        if let Some(ref desc) = rule.description {
            entries.push((super::MDC_DESCRIPTION, desc.clone()));
        }

        match rule.apply {
            ApplyMode::Always => {
                if let Some(ref patterns) = rule.apply_to {
                    if !patterns.is_empty() {
                        entries.push((super::MDC_GLOBS, patterns.join(", ")));
                    }
                }
                entries.push((super::MDC_ALWAYS_APPLY, "true".into()));
            }
            ApplyMode::Glob => {
                if let Some(ref patterns) = rule.apply_to {
                    entries.push((super::MDC_GLOBS, patterns.join(", ")));
                }
                entries.push((super::MDC_ALWAYS_APPLY, "false".into()));
            }
            ApplyMode::ModelDecision | ApplyMode::Manual => {
                entries.push((super::MDC_ALWAYS_APPLY, "false".into()));
            }
            _ => {}
        }

        let fm_refs: Vec<(&str, &str)> = entries.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let frontmatter = mdc_frontmatter(&fm_refs);
        files.push((
            CursorLayout::rule(&name),
            format!("{frontmatter}{content}").into(),
        ));
    }

    Ok(files)
}

// cast: hooks
// ref: https://cursor.com/docs/hooks

fn cast_hooks(manifest: &ThetaManifest) -> Result<Vec<CastFile>> {
    let key = theta_harness::HarnessTarget::Cursor.toml_key();
    let cfg: Option<CursorConfig> = manifest
        .harness_config(key)
        .map_err(|e| anyhow::anyhow!("[harness.{key}]: failed to parse: {e}"))?;

    let Some(cfg) = cfg else {
        return Ok(vec![]);
    };

    let Some(hooks) = cfg.hooks else {
        return Ok(vec![]);
    };

    let out = if let Some(map) = hooks.as_object() {
        let mut out = JsonMap::new();
        for (key, val) in map {
            out.insert(key.clone(), val.clone());
        }
        if out.is_empty() {
            return Ok(vec![]);
        }
        out
    } else {
        let mut out = JsonMap::new();
        out.insert("hooks".into(), hooks);
        out
    };

    let json = serde_json::to_string_pretty(&out)
        .map_err(|e| anyhow::anyhow!("failed to serialize hooks.json: {e}"))?;
    Ok(vec![(CursorLayout::hooks(), json.into())])
}

// cast: MCP tools

/// Build `.cursor/mcp.json` from `[tools]` + `[harness.cursor.mcp_extras]`.
/// theta-typed fields win on conflict with extras.
/// ref: <https://cursor.com/docs/mcp>
fn cast_mcp(manifest: &ThetaManifest) -> Result<Vec<CastFile>> {
    let key = theta_harness::HarnessTarget::Cursor.toml_key();
    let cfg: Option<CursorConfig> = manifest
        .harness_config(key)
        .map_err(|e| anyhow::anyhow!("[harness.{key}]: failed to parse: {e}"))?;

    let has_tools = manifest.tools.as_ref().is_some_and(|t| !t.is_empty());
    let has_extras = cfg.as_ref().is_some_and(|c| !c.mcp_extras.is_empty());
    if !has_tools && !has_extras {
        return Ok(vec![]);
    }

    let mut mcp_servers = JsonMap::new();

    if let Some(ref tools) = manifest.tools {
        for (name, tool) in tools {
            if !tool.enabled {
                continue;
            }
            let mut typed = JsonMap::new();
            if let Some(ref cmd) = tool.command {
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

            // merge extras (base) with typed (overlay, wins on conflict)
            let extras_map = cfg
                .as_ref()
                .and_then(|c| c.mcp_extras.get(name))
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            let mut sink = Vec::new();
            let merged = crate::common::merge_json_objects(
                extras_map,
                typed,
                &format!(".cursor/mcp.json --> {name}"),
                &mut sink,
            );
            mcp_servers.insert(name.clone(), JsonValue::Object(merged));
        }
    }

    // pure-extras-only servers (in [harness.cursor.mcp_extras] but not [tools])
    if let Some(ref c) = cfg {
        for (name, extras_val) in &c.mcp_extras {
            if mcp_servers.contains_key(name) {
                continue;
            }
            let extras_obj = extras_val.as_object().cloned().unwrap_or_else(JsonMap::new);
            mcp_servers.insert(name.clone(), JsonValue::Object(extras_obj));
        }
    }

    if mcp_servers.is_empty() {
        return Ok(vec![]);
    }

    let wrapper = serde_json::json!({ "mcpServers": JsonValue::Object(mcp_servers) });
    let json = serde_json::to_string_pretty(&wrapper)
        .map_err(|e| anyhow::anyhow!("failed to serialize .cursor/mcp.json: {e}"))?;
    Ok(vec![(CursorLayout::mcp(), json.into())])
}

// cast: skills
// ref: https://cursor.com/docs/skills

fn cast_skills(manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
    let Some(ref skills) = manifest.skills else {
        return Ok(vec![]);
    };
    let mut files = Vec::new();
    for name in skills.keys() {
        let skill_files = read_skill_dir_files(theta_dir, name, &CursorLayout::skills_dir())?;
        files.extend(skill_files);
    }
    Ok(files)
}

// cast: subagents
// ref: https://cursor.com/docs/subagents

fn cast_subagents(manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
    let Some(ref subagents) = manifest.subagents else {
        return Ok(vec![]);
    };

    let key = theta_harness::HarnessTarget::Cursor.toml_key();
    let cfg: Option<CursorConfig> = manifest
        .harness_config(key)
        .map_err(|e| anyhow::anyhow!("[harness.{key}]: failed to parse: {e}"))?;

    let mut files = Vec::new();
    for subagent in subagents {
        let resolved = ResolvedSubagent::load(subagent, theta_dir);

        let mut fm: Vec<(&str, serde_norway::Value)> = Vec::new();
        fm.push(("description", fm_str(resolved.description())));
        if let Some(m) = resolved.model() {
            fm.push(("model", fm_str(m)));
        }
        // cursor-specific fields from [harness.cursor.subagent.<name>]
        if let Some(ref cfg) = cfg {
            if let Some(extras) = cfg.subagent.get(&subagent.name) {
                if extras.get("readonly").and_then(serde_json::Value::as_bool) == Some(true) {
                    fm.push(("readonly", fm_bool(true)));
                }
                if extras
                    .get("is_background")
                    .and_then(serde_json::Value::as_bool)
                    == Some(true)
                {
                    fm.push(("is_background", fm_bool(true)));
                }
            }
        }
        let frontmatter = yaml_frontmatter(&fm)?;
        let body = resolved.body("")?;
        files.push((
            CursorLayout::agent(&subagent.name),
            format!("{frontmatter}{body}").into(),
        ));
    }

    Ok(files)
}

// orchestrator

pub(super) fn cast_files(manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>> {
    let mut files = Vec::new();

    files.push(cast_system_prompt(manifest, theta_dir)?);
    files.extend(cast_rules(manifest, theta_dir)?);
    files.extend(cast_hooks(manifest)?);
    files.extend(cast_mcp(manifest)?);
    files.extend(cast_skills(manifest, theta_dir)?);
    files.extend(cast_subagents(manifest, theta_dir)?);

    Ok(files)
}
