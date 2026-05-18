//! Codex CLI importer: codex-native files --> theta manifest.
//!
//! See `mod.rs` for the surface inventory and ref URLs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::common::{
    default_agent_name, import_skills_from_dir, json_to_toml_item_with_diagnostics,
    new_import_document, reorder_import_document, set_import_agent, toml_str_to_json,
};
use crate::{ImportOptions, ImportResult, Importer};
use anyhow::{Context, Result};
use theta_harness::layout::{CodexCliLayout, HarnessLayout};
use theta_schema::Diagnostic;

use super::CodexCli;

// codex subagent files are full config layers (per the docs they may carry any
// supported `config.toml` key). theta carves out four typed slots; everything
// else round-trips through `[harness.codex.subagent.<slug>]`. the carved-out
// keys live in `theta_static::CODEX_TYPED_AGENT_KEYS` (must match what `cast.rs`
// emits as theta-typed keys, so round-trip doesn't double-write).
//
// ref: https://developers.openai.com/codex/subagents#custom-agent-file-schema

impl Importer for CodexCli {
    fn import(&self, project_dir: &Path, opts: &ImportOptions) -> Result<ImportResult> {
        let mut doc = new_import_document();
        let mut extracted = Vec::new();
        let mut sources = Vec::new();
        let mut diags = Vec::new();

        // AGENTS.md is treated as opaque developer-instructions content. agent
        // identity is derived from the project directory; nothing is parsed
        // from the file body. codex concatenates AGENTS.md files from the
        // project root down to cwd, plus per-directory AGENTS.override.md;
        // theta only imports the project-root file and emits a hint for the
        // rest.
        // ref: <https://developers.openai.com/codex/guides/agents-md>
        set_import_agent(
            &mut doc,
            &default_agent_name(project_dir),
            "imported from codex-cli",
        );

        let agents_path = project_dir.join(CodexCliLayout::agents_md());
        if agents_path.is_file() {
            let content = fs_err::read_to_string(&agents_path)?;
            sources.push(CodexCliLayout::agents_md());
            if !content.trim().is_empty() {
                extracted.push((
                    PathBuf::from(theta_static::SYSTEM_FILE_NAME),
                    content.into(),
                ));
                theta_manifest::set_system_path(&mut doc);
            }
        } else {
            diags.push(Diagnostic::hint(
                "AGENTS.md",
                "not found - using default agent name",
            ));
        }

        // skills: .agents/skills/ (canonical cross-agent) + .codex/skills/
        // (legacy codex-only). canonical path wins on conflict.
        // ref: <https://developers.openai.com/codex/skills>
        let skill_results = import_skills_with_dedup(project_dir, &mut diags)?;
        for (name, cast_files, source_rel) in skill_results {
            theta_manifest::set_local_skill(&mut doc, &name);
            extracted.extend(cast_files);
            sources.push(source_rel);
        }

        // .codex/config.toml: split into typed [tools] (from [mcp_servers])
        // and opaque [harness.codex] (everything else).
        // ref: <https://developers.openai.com/codex/config-reference>
        let config = import_config_file(project_dir, &mut diags)?;
        sources.extend(config.sources);
        if let Some(tools_table) = config.tools_table {
            theta_manifest::set_section(&mut doc, "tools", tools_table)?;
        }
        let mut codex_table = config.codex_table;

        // .codex/hooks.json --> [harness.codex.hooks]. inline [hooks] in
        // config.toml is already captured under `codex_table` via the
        // top-level passthrough; the JSON file overlays only when present.
        // ref: <https://developers.openai.com/codex/hooks>
        let hooks = import_hooks_file(project_dir, &mut diags)?;
        sources.extend(hooks.sources);
        if let Some(hooks_item) = hooks.hooks_item {
            codex_table[CodexCliLayout::HOOKS_KEY] = hooks_item;
        }

        // .codex/agents/<name>.toml -> [[subagents]] + per-agent extras.
        // ref: <https://developers.openai.com/codex/subagents>
        let agents = import_agents_dir(project_dir, opts)?;
        sources.extend(agents.sources);
        if let Some(arr) = agents.subagents_array {
            theta_manifest::set_subagents(&mut doc, arr);
        }

        // overlay per-server MCP extras and per-agent subagent extras onto
        // `codex_table`. mirrors the claude pattern exactly.
        extend_codex_table_with_extras(
            &mut codex_table,
            theta_static::HARNESS_EXTRAS_TOOL_KEY,
            &config.extras_per_server,
            &mut diags,
            |name| {
                format!(
                    "{} --> {}.{name}",
                    CodexCliLayout::config().display(),
                    CodexCliLayout::MCP_ROOT_KEY,
                )
            },
        );
        extend_codex_table_with_extras(
            &mut codex_table,
            theta_static::HARNESS_EXTRAS_SUBAGENT_KEY,
            &agents.extras_per_agent,
            &mut diags,
            |name| CodexCliLayout::agent(name).display().to_string(),
        );

        // .codex/rules/*.rules: Starlark exec-policy, no portable equivalent.
        // emit a hint when detected; do not import.
        // ref: <https://developers.openai.com/codex/rules>
        let rules_dir = project_dir.join(CodexCliLayout::rules_dir());
        if rules_dir.is_dir() {
            let has_rules = fs_err::read_dir(&rules_dir)
                .map(|entries| {
                    entries
                        .filter_map(std::result::Result::ok)
                        .any(|e| e.path().extension().is_some_and(|ext| ext == "rules"))
                })
                .unwrap_or(false);
            if has_rules {
                diags.push(Diagnostic::hint(
                    ".codex/rules/",
                    "Codex exec-policy rules (.rules files) not imported - Starlark format has no portable theta equivalent",
                ));
            }
        }

        attach_harness_table(&mut doc, codex_table)?;
        reorder_import_document(&mut doc);

        Ok(ImportResult {
            document: doc,
            extracted_files: extracted,
            sources_read: sources,
            diagnostics: diags,
        })
    }
}

/// Hang the `[harness.codex]` table off `doc`, folding in the detected CLI
/// version. no-op when the table and version are both empty.
fn attach_harness_table(
    doc: &mut toml_edit::DocumentMut,
    mut codex_table: toml_edit::Table,
) -> Result<()> {
    let detected = crate::harnesses::version::detect_codex_cli_version();
    if codex_table.is_empty() && detected.is_none() {
        return Ok(());
    }
    if let Some(ref ver) = detected {
        codex_table["version"] = toml_edit::value(ver.clone());
    }
    let mut harness_table = toml_edit::Table::new();
    harness_table[theta_harness::HarnessTarget::CodexCli.toml_key()] =
        toml_edit::Item::Table(codex_table);
    theta_manifest::set_section(doc, "harness", harness_table)?;
    Ok(())
}

/// Fold generic JSON extras into `codex_table[key]` as a nested table keyed by
/// `<inner_name>`. used for both `[harness.codex.tool.*]` MCP extras and
/// `[harness.codex.subagent.*]` subagent extras.
fn extend_codex_table_with_extras<F>(
    codex_table: &mut toml_edit::Table,
    key: &str,
    extras: &BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    diags: &mut Vec<Diagnostic>,
    mut context_for: F,
) where
    F: FnMut(&str) -> String,
{
    if extras.is_empty() {
        return;
    }
    let mut table = toml_edit::Table::new();
    table.set_implicit(true);
    for (name, payload) in extras {
        let value = serde_json::Value::Object(payload.clone());
        let context = context_for(name);
        table[name.as_str()] = json_to_toml_item_with_diagnostics(&value, &context, diags);
    }
    codex_table[key] = toml_edit::Item::Table(table);
}

// skills

/// Walk both `.agents/skills/` (canonical) and `.codex/skills/` (legacy). on
/// name collision, the canonical entry wins and the legacy duplicate is
/// dropped with a hint.
fn import_skills_with_dedup(
    project_dir: &Path,
    diags: &mut Vec<Diagnostic>,
) -> Result<Vec<(String, Vec<crate::CastFile>, PathBuf)>> {
    let mut out: Vec<(String, Vec<crate::CastFile>, PathBuf)> = Vec::new();
    let mut seen: BTreeMap<String, &'static str> = BTreeMap::new();

    // canonical first
    for (label, dir) in [
        ("cross-agent", CodexCliLayout::cross_agent_skills_dir()),
        ("codex-specific", CodexCliLayout::codex_skills_dir()),
    ] {
        let results = import_skills_from_dir(project_dir, &dir)?;
        for (name, cast_files, source_rel) in results {
            if let Some(prev_label) = seen.get(&name) {
                diags.push(Diagnostic::hint(
                    source_rel.display().to_string(),
                    format!(
                        "skill '{name}' already imported from {prev_label} location; dropping {label} duplicate"
                    ),
                ));
                continue;
            }
            seen.insert(name.clone(), label);
            out.push((name, cast_files, source_rel));
        }
    }
    Ok(out)
}

// .codex/config.toml split

struct ConfigImport {
    tools_table: Option<toml_edit::Table>,
    extras_per_server: BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    codex_table: toml_edit::Table,
    sources: Vec<PathBuf>,
}

/// Parse `.codex/config.toml`. A missing file is not an error. Splits the
/// document into typed `[tools]` (from `[mcp_servers.*]`) and an opaque
/// `[harness.codex]` table for everything else.
///
/// theta-typed MCP keys (`command`, `args`, `env`, `url`, `headers`) go to
/// `[tools]`. all other per-server fields go to
/// `[harness.codex.tool.<name>]` for lossless round-trip.
///
/// ref: <https://developers.openai.com/codex/mcp>
fn import_config_file(project_dir: &Path, diags: &mut Vec<Diagnostic>) -> Result<ConfigImport> {
    let mut out = ConfigImport {
        tools_table: None,
        extras_per_server: BTreeMap::new(),
        codex_table: toml_edit::Table::new(),
        sources: Vec::new(),
    };

    let config_path = project_dir.join(CodexCliLayout::config());
    if !config_path.is_file() {
        return Ok(out);
    }

    let raw = fs_err::read_to_string(&config_path)?;
    out.sources.push(CodexCliLayout::config());

    let json = toml_str_to_json(&raw)
        .with_context(|| format!("failed to parse {}", config_path.display()))?;

    let Some(map) = json.as_object() else {
        return Ok(out);
    };

    // [mcp_servers.<name>] --> [tools] + per-server extras
    if let Some(serde_json::Value::Object(servers)) = map.get(CodexCliLayout::MCP_ROOT_KEY) {
        let mut tools_table = toml_edit::Table::new();
        for (name, server) in servers {
            let Some(server_obj) = server.as_object() else {
                continue;
            };
            let mut tool = toml_edit::Table::new();

            // codex infers transport from `command` (stdio) vs `url` (HTTP).
            // there is no explicit `type` enum on per-server entries.
            // ref: <https://developers.openai.com/codex/mcp>
            if let Some(cmd) = server_obj
                .get(theta_static::MCP_KEY_COMMAND)
                .and_then(|v| v.as_str())
            {
                let mut arr = toml_edit::Array::new();
                arr.push(cmd);
                tool[theta_static::MCP_KEY_COMMAND] = toml_edit::value(arr);
            }
            if let Some(args) = server_obj
                .get(theta_static::MCP_KEY_ARGS)
                .and_then(|v| v.as_array())
            {
                let arr: toml_edit::Array = args.iter().filter_map(|a| a.as_str()).collect();
                if !arr.is_empty() {
                    tool[theta_static::MCP_KEY_ARGS] = toml_edit::value(arr);
                }
            }
            if let Some(url) = server_obj
                .get(theta_static::MCP_KEY_URL)
                .and_then(|v| v.as_str())
            {
                tool[theta_static::MCP_KEY_URL] = toml_edit::value(url);
            }
            if let Some(env) = server_obj
                .get(theta_static::MCP_KEY_ENV)
                .and_then(|v| v.as_object())
                .filter(|e| !e.is_empty())
            {
                let mut env_table = toml_edit::InlineTable::new();
                for (k, v) in env {
                    if let Some(s) = v.as_str() {
                        env_table.insert(k, s.into());
                    }
                }
                tool[theta_static::MCP_KEY_ENV] = toml_edit::value(env_table);
            }
            // codex MCP HTTP headers live under `http_headers`, not `headers`.
            // ref: <https://developers.openai.com/codex/config-reference>
            if let Some(hdr_obj) = server_obj
                .get(CodexCliLayout::MCP_HTTP_HEADERS_KEY)
                .and_then(|v| v.as_object())
                .filter(|h| !h.is_empty())
            {
                let mut hdr_table = toml_edit::InlineTable::new();
                for (k, v) in hdr_obj {
                    if let Some(s) = v.as_str() {
                        hdr_table.insert(k, s.into());
                    }
                }
                // theta [tools] uses `headers` (portable); codex uses
                // `http_headers` on the wire. typed slot maps both sides.
                tool[theta_static::MCP_KEY_HEADERS] = toml_edit::value(hdr_table);
            }

            // Codex CLI supports `enabled = false` to register a server as
            // inactive. Round-trips into theta's [tools.<name>].enabled.
            // ref: <https://developers.openai.com/codex/mcp>
            if let Some(false) = server_obj
                .get(CodexCliLayout::MCP_ENABLED_KEY)
                .and_then(serde_json::Value::as_bool)
            {
                tool[CodexCliLayout::MCP_ENABLED_KEY] = toml_edit::value(false);
            }

            // extras: every per-server key not consumed by theta-typed
            // [tools] above round-trips through [harness.codex.tool.<name>].
            // ref: <https://developers.openai.com/codex/config-reference>
            let mut extras = serde_json::Map::new();
            for (k, v) in server_obj {
                if CodexCliLayout::TYPED_MCP_KEYS.contains(&k.as_str()) {
                    continue;
                }
                extras.insert(k.clone(), v.clone());
            }
            if !extras.is_empty() {
                out.extras_per_server.insert(name.clone(), extras);
            }

            tools_table[name] = toml_edit::Item::Table(tool);
        }
        if !tools_table.is_empty() {
            out.tools_table = Some(tools_table);
        }
    }

    // every other top-level key --> [harness.codex]
    for (key, value) in map {
        if key == CodexCliLayout::MCP_ROOT_KEY {
            continue;
        }
        let context = format!(
            "harness.{}.{key}",
            theta_harness::HarnessTarget::CodexCli.toml_key()
        );
        out.codex_table[key.as_str()] = json_to_toml_item_with_diagnostics(value, &context, diags);
    }

    Ok(out)
}

// .codex/hooks.json

struct HooksImport {
    hooks_item: Option<toml_edit::Item>,
    sources: Vec<PathBuf>,
}

fn import_hooks_file(project_dir: &Path, diags: &mut Vec<Diagnostic>) -> Result<HooksImport> {
    let mut out = HooksImport {
        hooks_item: None,
        sources: Vec::new(),
    };

    let hooks_path = project_dir.join(CodexCliLayout::hooks());
    if !hooks_path.is_file() {
        return Ok(out);
    }

    let raw = fs_err::read_to_string(&hooks_path)?;
    out.sources.push(CodexCliLayout::hooks());

    let json = crate::common::parse_jsonc_value(&raw, &hooks_path)?;
    let ctx = format!(
        "harness.{}.hooks",
        theta_harness::HarnessTarget::CodexCli.toml_key()
    );
    out.hooks_item = Some(json_to_toml_item_with_diagnostics(&json, &ctx, diags));
    Ok(out)
}

// .codex/agents/<name>.toml

struct AgentsImport {
    subagents_array: Option<toml_edit::ArrayOfTables>,
    extras_per_agent: BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    sources: Vec<PathBuf>,
}

fn import_agents_dir(project_dir: &Path, opts: &ImportOptions) -> Result<AgentsImport> {
    let mut out = AgentsImport {
        subagents_array: None,
        extras_per_agent: BTreeMap::new(),
        sources: Vec::new(),
    };

    let agents_dir = project_dir.join(CodexCliLayout::agents_dir());
    if !agents_dir.is_dir() {
        return Ok(out);
    }

    let mut entries: Vec<_> = fs_err::read_dir(&agents_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        .collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);

    if entries.is_empty() {
        return Ok(out);
    }

    let mut arr = toml_edit::ArrayOfTables::new();

    for entry in entries {
        let path = entry.path();
        let (slug, table, extras) = import_one_agent_file(&path, opts, project_dir)?;
        out.sources.push(CodexCliLayout::agent(&slug));
        if !extras.is_empty() {
            out.extras_per_agent.insert(slug, extras);
        }
        arr.push(table);
    }

    out.subagents_array = Some(arr);
    Ok(out)
}

/// Parse one `.codex/agents/<name>.toml` into a `[[subagents]]` table plus
/// extras. Frontmatter `name` wins over filename stem.
///
/// codex subagent files are full config layers - they may contain any
/// supported `config.toml` key. theta carves out four typed slots and routes
/// everything else through [harness.codex.subagent.<slug>] extras.
/// ref: <https://developers.openai.com/codex/subagents#custom-agent-file-schema>
fn import_one_agent_file(
    path: &Path,
    opts: &ImportOptions,
    manifest_dir: &Path,
) -> Result<(
    String,
    toml_edit::Table,
    serde_json::Map<String, serde_json::Value>,
)> {
    let stem = crate::common::file_stem_str(path)?;
    let raw = fs_err::read_to_string(path)?;
    let agent_json =
        toml_str_to_json(&raw).with_context(|| format!("failed to parse {}", path.display()))?;

    let mut table = toml_edit::Table::new();
    let mut extras = serde_json::Map::new();

    let obj = agent_json.as_object().cloned().unwrap_or_default();
    let agent_name = obj
        .get(CodexCliLayout::AGENT_KEY_NAME)
        .and_then(|v| v.as_str())
        .unwrap_or(&stem)
        .to_string();
    table[CodexCliLayout::AGENT_KEY_NAME] = toml_edit::value(&agent_name);

    // description is required per docs; if missing in the file we still
    // record the name and emit a hint rather than fabricating one.
    // ref: <https://developers.openai.com/codex/subagents#custom-agent-file-schema>
    if let Some(desc) = obj
        .get(CodexCliLayout::AGENT_KEY_DESCRIPTION)
        .and_then(|v| v.as_str())
    {
        table[CodexCliLayout::AGENT_KEY_DESCRIPTION] = toml_edit::value(desc);
    }

    // developer_instructions is the body of the codex subagent. theta
    // externalizes it to a prompt file the same way claude / copilot do.
    if let Some(instructions) = obj
        .get(CodexCliLayout::AGENT_KEY_DEV_INSTRUCTIONS)
        .and_then(|v| v.as_str())
    {
        if !instructions.trim().is_empty() {
            let rel = crate::common::write_subagent_prompt(
                &opts.subagent_prompts_dir,
                &agent_name,
                instructions,
                opts.force_overwrite,
                manifest_dir,
            )?;
            table["prompt_path"] = toml_edit::value(&rel);
        }
    }

    if let Some(model) = obj
        .get(CodexCliLayout::AGENT_KEY_MODEL)
        .and_then(|v| v.as_str())
    {
        table[CodexCliLayout::AGENT_KEY_MODEL] = toml_edit::value(model);
    }

    // extras: every top-level key in the agent TOML not consumed by the four
    // theta-typed slots above lands in [harness.codex.subagent.<slug>]
    // verbatim. fields like nickname_candidates, sandbox_mode,
    // model_reasoning_effort, nested [mcp_servers.*], [permissions.*],
    // [skills.config], [features], etc. all round-trip here.
    for (k, v) in &obj {
        if CodexCliLayout::TYPED_AGENT_KEYS.contains(&k.as_str()) {
            continue;
        }
        extras.insert(k.clone(), v.clone());
    }

    let slug = theta_static::kebab_case(&agent_name);
    Ok((slug, table, extras))
}
