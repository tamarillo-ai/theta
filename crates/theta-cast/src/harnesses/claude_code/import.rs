use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::common::{
    default_agent_name, import_skills_from_dir, json_to_toml_item_with_diagnostics,
    new_import_document, parse_frontmatter, reorder_import_document, set_import_agent,
};
use crate::{ImportOptions, ImportResult, Importer};
use anyhow::Result;
use theta_harness::layout::{ClaudeCodeLayout, HarnessLayout};
use theta_schema::Diagnostic;

use super::ClaudeCode;

/// Frontmatter keys consumed as typed `[[subagents]]` fields. Every other key
/// in a `.claude/agents/<name>.md` frontmatter is preserved in
/// `[harness.claude_code.subagent.<slug>]` so it round-trips on cast.
///
/// The shared `THETA_TYPED_AGENT_KEYS` set is `name`/`description`/`model`/`tools`;
/// Claude also lets theta type `prompt`/`skills`, so those are filtered too
/// to avoid duplicate emission.
///
/// Claude-specific fields that round-trip through extras: `permissionMode`,
/// `mcpServers`, `hooks`, `memory`, `background`, `effort`, `isolation`,
/// `color`, `initialPrompt`, `disallowedTools`, `maxTurns`.
///
/// ref: <https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields>
const THETA_CLAUDE_AGENT_EXTRA_FILTER: &[&str] =
    &["name", "description", "model", "tools", "prompt", "skills"];

impl Importer for ClaudeCode {
    fn import(&self, project_dir: &Path, opts: &ImportOptions) -> Result<ImportResult> {
        let mut doc = new_import_document();
        let mut extracted = Vec::new();
        let mut sources = Vec::new();
        let mut diags = Vec::new();

        // `CLAUDE.md` (or `.claude/CLAUDE.md` as alternate) is treated as
        // opaque system prompt content. agent identity is derived from the
        // project directory: parsing the file's first H1 or paragraph as
        // identity strips user-authored content and re-emits a synthetic
        // block on cast.
        // ref: <https://code.claude.com/docs/en/memory#choose-where-to-put-claude-md-files>
        set_import_agent(
            &mut doc,
            &default_agent_name(project_dir),
            "imported from claude-code",
        );

        let claude_md_primary = project_dir.join(ClaudeCodeLayout::system_prompt());
        let claude_md_alt = project_dir.join(ClaudeCodeLayout::system_prompt_alt());
        let system_path = if claude_md_primary.is_file() {
            Some(ClaudeCodeLayout::system_prompt())
        } else if claude_md_alt.is_file() {
            diags.push(Diagnostic::hint(
                ClaudeCodeLayout::system_prompt_alt().display().to_string(),
                "alternate CLAUDE.md location - cast will write to ./CLAUDE.md instead",
            ));
            Some(ClaudeCodeLayout::system_prompt_alt())
        } else {
            diags.push(Diagnostic::hint(
                "CLAUDE.md",
                "not found - using default agent name",
            ));
            None
        };

        if let Some(rel) = system_path {
            let abs = project_dir.join(&rel);
            let content = fs_err::read_to_string(&abs)?;
            sources.push(rel);
            if !content.trim().is_empty() {
                extracted.push((
                    PathBuf::from(theta_static::SYSTEM_FILE_NAME),
                    content.into(),
                ));
                theta_manifest::set_system_path(&mut doc);
            }
        }

        // cross-read: `AGENTS.md` (and any future shared instruction files).
        // claude reads `AGENTS.md` only when CLAUDE.md imports it via
        // `@AGENTS.md`, but many repos use it as the primary always-on
        // instruction file across harnesses. cross-read is opt-in to avoid
        // silent duplication on subsequent round-trips.
        // ref: <https://code.claude.com/docs/en/memory#agents-md>
        if opts.cross_read {
            for rel in ClaudeCodeLayout::CROSS_READ_SYSTEM_PROMPT_FILES {
                let path = project_dir.join(rel);
                let Some(content) = crate::common::read_cross_read_file(&path) else {
                    continue;
                };
                diags.push(crate::common::cross_read_hint(rel, "another harness"));
                sources.push(path);
                crate::common::append_cross_read_to_system_prompt(
                    &mut doc,
                    &mut extracted,
                    &content,
                );
            }
        }

        // `.claude/rules/**/*.md` to `[instructions.rules]`. claude scans the
        // rules dir recursively and treats subdirectories as first-class
        // organization; theta preserves that structure as path-qualified
        // names (`frontend/api-design` to
        // `[instructions.rules."frontend/api-design"]`).
        // ref: <https://code.claude.com/docs/en/memory#organize-rules-with-claude-rules>
        let rules = import_rules_dir(project_dir)?;
        sources.extend(rules.sources);
        extracted.extend(rules.extracted);
        if let Some(rules_table) = rules.rules_table {
            theta_manifest::set_rules_section(&mut doc, rules_table);
        }

        // `.claude/skills/*/SKILL.md` to `[skills]`. each skill directory is
        // copied byte-for-byte by `import_skills_from_dir`; theta does not
        // parse SKILL.md frontmatter, so claude-specific fields like
        // `disable-model-invocation`, `allowed-tools`, `paths`,
        // `context: fork`, etc. round-trip losslessly.
        // ref: <https://code.claude.com/docs/en/skills>
        let skill_results = import_skills_from_dir(project_dir, &ClaudeCodeLayout::skills_dir())?;
        for (name, cast_files, source_rel) in skill_results {
            theta_manifest::set_local_skill(&mut doc, &name);
            extracted.extend(cast_files);
            sources.push(source_rel);
        }

        // `.claude/agents/*.md` to `[[subagents]]` plus per-agent extras keyed
        // by kebab-cased filename stem under `[harness.claude_code.subagent.<slug>]`.
        // ref: <https://code.claude.com/docs/en/sub-agents#write-subagent-files>
        let agents = import_agents_dir(project_dir, opts)?;
        sources.extend(agents.sources);
        if let Some(arr) = agents.subagents_array {
            theta_manifest::set_subagents(&mut doc, arr);
        }

        // `.mcp.json` `mcpServers` to `[tools]` plus per-server extras keyed
        // by server name under `[harness.claude_code.tool.<name>]`.
        // theta-typed keys win on cast; the extras blob preserves `oauth`,
        // `headersHelper`, `alwaysLoad`, etc.
        // ref: <https://code.claude.com/docs/en/mcp#project-scope>
        let mcp = import_mcp_file(project_dir)?;
        sources.extend(mcp.sources);
        if let Some(tools_table) = mcp.tools_table {
            theta_manifest::set_section(&mut doc, "tools", tools_table)?;
        }

        // `.claude/settings.json` to base `[harness.claude_code]` table. every
        // top-level key passes through verbatim with its native camelCase
        // name; theta does not transform sandbox/hooks/permissions/etc.
        // ref: <https://code.claude.com/docs/en/settings>
        let (mut cc_table, settings_sources) = import_settings_file(project_dir, &mut diags)?;
        sources.extend(settings_sources);

        // overlay per-server MCP extras and per-agent subagent extras onto `cc_table`.
        extend_cc_table_with_extras(
            &mut cc_table,
            "tool",
            &mcp.extras_per_server,
            &mut diags,
            |name| format!("{} - {name}", ClaudeCodeLayout::mcp().display()),
        );
        extend_cc_table_with_extras(
            &mut cc_table,
            "subagent",
            &agents.extras_per_agent,
            &mut diags,
            |name| ClaudeCodeLayout::agent(name).display().to_string(),
        );

        attach_harness_table(&mut doc, cc_table)?;
        reorder_import_document(&mut doc);

        Ok(ImportResult {
            document: doc,
            extracted_files: extracted,
            sources_read: sources,
            diagnostics: diags,
        })
    }
}

/// Hang the `[harness.claude_code]` table off `doc`, folding in the detected
/// CLI version. no-op when the table and version are both empty.
fn attach_harness_table(
    doc: &mut toml_edit::DocumentMut,
    mut cc_table: toml_edit::Table,
) -> Result<()> {
    let detected = crate::harnesses::version::detect_claude_code_version();
    if cc_table.is_empty() && detected.is_none() {
        return Ok(());
    }
    if let Some(ref ver) = detected {
        cc_table["version"] = toml_edit::value(ver.clone());
    }
    let mut harness_table = toml_edit::Table::new();
    harness_table[theta_harness::HarnessTarget::ClaudeCode.toml_key()] =
        toml_edit::Item::Table(cc_table);
    theta_manifest::set_section(doc, "harness", harness_table)?;
    Ok(())
}

/// Fold generic JSON extras into `cc_table[key]` as a nested table keyed by
/// `<inner_name>`. used for both `[harness.claude_code.tool.*]` MCP extras
/// and `[harness.claude_code.subagent.*]` subagent frontmatter extras.
fn extend_cc_table_with_extras<F>(
    cc_table: &mut toml_edit::Table,
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
    cc_table[key] = toml_edit::Item::Table(table);
}

// rules

struct RulesImport {
    rules_table: Option<toml_edit::Table>,
    extracted: Vec<crate::CastFile>,
    sources: Vec<PathBuf>,
}

/// Recursively collect `*.md` files under `dir`, returning `(stem, path)` pairs
/// where stem is the path-qualified rule name relative to `base`.
/// (e.g. `"frontend/api-design"`). mirrors `copilot::collect_instruction_files`.
fn collect_rule_files(dir: &Path, base: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = fs_err::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.filter_map(std::result::Result::ok).collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_rule_files(&path, base, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let Some(rel) = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_str()
                .map(String::from)
            else {
                continue;
            };
            if let Some(stem) = rel.strip_suffix(".md") {
                let stem = stem.trim_end_matches('/');
                if !stem.is_empty() && stem.split('/').all(|s| !s.is_empty()) {
                    out.push((stem.to_string(), path));
                }
            }
        }
    }
}

fn import_rules_dir(project_dir: &Path) -> Result<RulesImport> {
    let mut out = RulesImport {
        rules_table: None,
        extracted: Vec::new(),
        sources: Vec::new(),
    };

    let rules_dir = project_dir.join(ClaudeCodeLayout::rules_dir());
    if !rules_dir.is_dir() {
        return Ok(out);
    }

    let mut rule_files = Vec::new();
    collect_rule_files(&rules_dir, &rules_dir, &mut rule_files);

    if rule_files.is_empty() {
        return Ok(out);
    }

    let mut rules_table = toml_edit::Table::new();

    for (stem, path) in &rule_files {
        let content = fs_err::read_to_string(path)?;
        out.sources.push(ClaudeCodeLayout::rule(stem));

        let parsed = parse_frontmatter(&content);
        let paths = parsed.get_str_list("paths");

        let mut rule = toml_edit::Table::new();
        let rule_rel = theta_static::ThetaProjectLayout::rule_rel(stem);
        rule["src"] = toml_edit::value(rule_rel.as_str());

        if let Some(ref pats) = paths {
            rule["apply"] = toml_edit::value("glob");
            let mut arr = toml_edit::Array::new();
            for p in pats {
                arr.push(p.as_str());
            }
            rule["apply_to"] = toml_edit::value(arr);
        } else {
            rule["apply"] = toml_edit::value("always");
        }

        rules_table[stem.as_str()] = toml_edit::Item::Table(rule);
        out.extracted
            .push((PathBuf::from(&rule_rel), parsed.content.into()));
    }

    out.rules_table = Some(rules_table);
    Ok(out)
}

// MCP

struct McpImportOutput {
    tools_table: Option<toml_edit::Table>,
    extras_per_server: BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    sources: Vec<PathBuf>,
}

/// Parse `.mcp.json` — a missing file is not an error.
///
/// theta-typed keys (`type`, `command`, `args`, `env`, `url`, `headers`) go to
/// `[tools]`. everything else (`oauth`, `headersHelper`, `alwaysLoad`, etc.)
/// goes to `[harness.claude_code.tool.<name>]` for lossless round-trip.
/// ref: <https://code.claude.com/docs/en/mcp#environment-variable-expansion-in-mcp-json>
fn import_mcp_file(project_dir: &Path) -> Result<McpImportOutput> {
    let mut out = McpImportOutput {
        tools_table: None,
        extras_per_server: BTreeMap::new(),
        sources: Vec::new(),
    };

    let mcp_path = project_dir.join(ClaudeCodeLayout::mcp());
    if !mcp_path.is_file() {
        return Ok(out);
    }

    let raw = fs_err::read_to_string(&mcp_path)?;
    out.sources.push(ClaudeCodeLayout::mcp());

    let json = crate::common::parse_jsonc_value(&raw, &mcp_path)?;

    let Some(servers) = json
        .get(ClaudeCodeLayout::MCP_ROOT_KEY)
        .and_then(|v| v.as_object())
    else {
        return Ok(out);
    };

    let mut tools_table = toml_edit::Table::new();

    for (name, server) in servers {
        let mut tool = toml_edit::Table::new();

        // `type` defaults to stdio when `command` is set, http otherwise.
        // claude also accepts `streamable-http` as an alias for `http`.

        let transport = server
            .get(theta_static::MCP_KEY_TYPE)
            .and_then(|v| v.as_str())
            .and_then(|s| theta_static::McpTransport::from_str(s).ok())
            .unwrap_or_else(|| {
                if server.get(theta_static::MCP_KEY_COMMAND).is_some() {
                    theta_static::McpTransport::Stdio
                } else {
                    theta_static::McpTransport::Http
                }
            });

        match transport {
            theta_static::McpTransport::Stdio => {
                if let Some(cmd) = server
                    .get(theta_static::MCP_KEY_COMMAND)
                    .and_then(|v| v.as_str())
                {
                    let mut arr = toml_edit::Array::new();
                    arr.push(cmd);
                    tool[theta_static::MCP_KEY_COMMAND] = toml_edit::value(arr);
                }
                if let Some(args) = server
                    .get(theta_static::MCP_KEY_ARGS)
                    .and_then(|v| v.as_array())
                {
                    let arr: toml_edit::Array = args.iter().filter_map(|a| a.as_str()).collect();
                    if !arr.is_empty() {
                        tool[theta_static::MCP_KEY_ARGS] = toml_edit::value(arr);
                    }
                }
            }
            theta_static::McpTransport::Http | theta_static::McpTransport::Sse => {
                if let Some(url) = server
                    .get(theta_static::MCP_KEY_URL)
                    .and_then(|v| v.as_str())
                {
                    tool[theta_static::MCP_KEY_URL] = toml_edit::value(url);
                }
                if let Some(headers) = server
                    .get(theta_static::MCP_KEY_HEADERS)
                    .and_then(|v| v.as_object())
                    .filter(|h| !h.is_empty())
                {
                    let mut hdr_table = toml_edit::InlineTable::new();
                    for (k, v) in headers {
                        if let Some(s) = v.as_str() {
                            hdr_table.insert(k, s.into());
                        }
                    }
                    tool[theta_static::MCP_KEY_HEADERS] = toml_edit::value(hdr_table);
                }
            }
            _ => {
                unreachable!();
            }
        }

        if let Some(env) = server
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

        // extras: every server-level field not in `THETA_TYPED_MCP_KEYS` rounds
        // back through `[harness.claude_code.tool.<name>]`.
        if let Some(server_obj) = server.as_object() {
            let mut extras = serde_json::Map::new();
            for (k, v) in server_obj {
                if !theta_static::THETA_TYPED_MCP_KEYS.contains(&k.as_str()) {
                    extras.insert(k.clone(), v.clone());
                }
            }
            if !extras.is_empty() {
                out.extras_per_server.insert(name.clone(), extras);
            }
        }

        tools_table[name] = toml_edit::Item::Table(tool);
    }

    if !tools_table.is_empty() {
        out.tools_table = Some(tools_table);
    }
    Ok(out)
}

// subagents

struct AgentsImport {
    subagents_array: Option<toml_edit::ArrayOfTables>,
    /// Per-subagent non-typed frontmatter for `[harness.claude_code.subagent.<slug>]`.
    extras_per_agent: BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    sources: Vec<PathBuf>,
}

/// Parse one `.claude/agents/<name>.md` into a `[[subagents]]` table plus extras.
/// Frontmatter `name` wins over filename stem (per docs, filename does not need
/// to match `name`). Description is optional — no synthetic fallback.
fn import_one_agent_file(
    path: &Path,
    opts: &crate::ImportOptions,
    manifest_dir: &Path,
) -> Result<(
    String,
    toml_edit::Table,
    serde_json::Map<String, serde_json::Value>,
)> {
    let stem = crate::common::file_stem_str(path)?;
    let content = fs_err::read_to_string(path)?;
    let parsed = parse_frontmatter(&content);

    let mut table = toml_edit::Table::new();
    let agent_name = parsed.get_str("name").unwrap_or(&stem).to_string();
    table["name"] = toml_edit::value(&agent_name);

    // claude marks `description` as recommended-but-optional in subagent
    // frontmatter. theta-schema models it as `Option<String>`, so only emit
    // when the source had one - cast checks the field's presence to decide
    // whether to write the line back.
    // ref: <https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields>
    if let Some(desc) = parsed.get_str("description") {
        table["description"] = toml_edit::value(desc);
    }

    let body = parsed.content.clone();
    if !body.trim().is_empty() {
        let rel = crate::common::write_subagent_prompt(
            &opts.subagent_prompts_dir,
            &stem,
            &body,
            opts.force_overwrite,
            manifest_dir,
        )?;
        table["prompt_path"] = toml_edit::value(&rel);
    }

    if let Some(model) = parsed.get_str("model") {
        table["model"] = toml_edit::value(model);
    }

    // claude accepts `tools` as either a yaml sequence or a comma-separated
    // string; the docs use the comma form for inline frontmatter. split on
    // commas so `tools = ["Read", "Glob", "Grep"]` lands in theta.toml.
    // ref: <https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields>
    if let Some(raw) = parsed.get_str_list("tools") {
        let tools: Vec<String> = raw
            .iter()
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !tools.is_empty() {
            let mut tools_arr = toml_edit::Array::new();
            for tool in &tools {
                tools_arr.push(tool.as_str());
            }
            table["tools"] = toml_edit::value(tools_arr);
        }
    }

    if let Some(skills_list) = parsed.get_str_list("skills") {
        if !skills_list.is_empty() {
            let mut skills_arr = toml_edit::Array::new();
            for s in &skills_list {
                skills_arr.push(s.as_str());
            }
            table["skills"] = toml_edit::value(skills_arr);
        }
    }

    // extras: every frontmatter field not consumed by theta-typed [[subagents]]
    // above lands in `[harness.claude_code.subagent.<slug>]` verbatim. fields
    // like permissionMode, mcpServers, hooks, memory, background, effort,
    // isolation, color, initialPrompt, disallowedTools all round-trip here.
    let mut extras = serde_json::Map::new();
    for (k, v) in &parsed.data {
        if THETA_CLAUDE_AGENT_EXTRA_FILTER.contains(&k.as_str()) {
            continue;
        }
        extras.insert(k.clone(), v.clone());
    }

    Ok((stem, table, extras))
}

fn import_agents_dir(project_dir: &Path, opts: &crate::ImportOptions) -> Result<AgentsImport> {
    let mut out = AgentsImport {
        subagents_array: None,
        extras_per_agent: BTreeMap::new(),
        sources: Vec::new(),
    };

    let agents_dir = project_dir.join(ClaudeCodeLayout::agents_dir());
    if !agents_dir.is_dir() {
        return Ok(out);
    }

    let mut entries: Vec<_> = fs_err::read_dir(&agents_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);

    if entries.is_empty() {
        return Ok(out);
    }

    let mut arr = toml_edit::ArrayOfTables::new();
    for entry in entries {
        let path = entry.path();
        let stem = crate::common::file_stem_str(&path)?;
        out.sources.push(ClaudeCodeLayout::agent(&stem));

        let (key, table, extras) = import_one_agent_file(&path, opts, project_dir)?;
        if !extras.is_empty() {
            let extras_key = theta_static::kebab_case(&key);
            out.extras_per_agent.insert(extras_key, extras);
        }
        arr.push(table);
    }

    if !arr.is_empty() {
        out.subagents_array = Some(arr);
    }
    Ok(out)
}

// settings.json

/// Read `.claude/settings.json` and build a base `[harness.claude_code]` table.
/// Every top-level key passes through verbatim; theta types a few slots on
/// `ClaudeCodeConfig` for documentation but treats them as opaque blobs.
/// ref: <https://code.claude.com/docs/en/settings#available-settings>
fn import_settings_file(
    project_dir: &Path,
    diags: &mut Vec<Diagnostic>,
) -> Result<(toml_edit::Table, Vec<PathBuf>)> {
    let mut cc_table = toml_edit::Table::new();
    let mut sources = Vec::new();

    let settings_path = project_dir.join(ClaudeCodeLayout::settings());
    if !settings_path.is_file() {
        return Ok((cc_table, sources));
    }

    let raw = fs_err::read_to_string(&settings_path)?;
    sources.push(ClaudeCodeLayout::settings());

    let json = crate::common::parse_jsonc_map(&raw, &settings_path)?;

    // each top-level key passes through verbatim with its native camelCase name.
    // claude-spec settings keys (sandbox, hooks, permissions, enabledPlugins,
    // autoMode, viewMode, attribution, worktree, etc.) all land in either the
    // typed slot of the same name on `ClaudeCodeConfig` or in the `extra`
    // flatten map - both routes round-trip losslessly.
    // ref: <https://code.claude.com/docs/en/settings#available-settings>
    for (key, value) in &json {
        let context = format!(
            "harness.{}.{key}",
            theta_harness::HarnessTarget::ClaudeCode.toml_key()
        );
        cc_table[key.as_str()] = json_to_toml_item_with_diagnostics(value, &context, diags);
    }

    Ok((cc_table, sources))
}
