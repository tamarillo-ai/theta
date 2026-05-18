use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::ImportResult;
use crate::common::{
    default_agent_name, import_skills_from_dir, json_to_toml_item,
    json_to_toml_item_with_diagnostics, new_import_document, parse_frontmatter,
    reorder_import_document, set_import_agent,
};
use anyhow::Result;
use theta_harness::layout::{CopilotLayout, HarnessLayout};
use theta_schema::Diagnostic;

type HookMap = serde_json::Map<String, serde_json::Value>;
type HookImportOutput = (HookMap, Vec<PathBuf>, Vec<Diagnostic>);

pub(super) fn import(project_dir: &Path, opts: &crate::ImportOptions) -> Result<ImportResult> {
    let mut doc = new_import_document();
    let mut extracted: Vec<crate::CastFile> = Vec::new();
    let mut sources = Vec::new();
    let mut diags = Vec::new();

    // .github/copilot-instructions.md --> identity + [instructions.system]
    // ref: https://code.visualstudio.com/docs/copilot/customization/custom-instructions
    let sys = import_system_prompt(project_dir)?;
    sources.extend(sys.sources);
    diags.extend(sys.diags);
    set_import_agent(&mut doc, &sys.agent_name, &sys.agent_description);
    if let Some((path, body)) = sys.extracted_file {
        extracted.push((path, body.into()));
        theta_manifest::set_system_path(&mut doc);
    }
    // cross-read: AGENTS.md, CLAUDE.md, .claude/CLAUDE.md, CLAUDE.local.md
    // these are always-on instruction files that copilot discovers from other harness locations
    // ref: https://code.visualstudio.com/docs/copilot/customization/custom-instructions#_types-of-instruction-files
    if opts.cross_read {
        for rel in CopilotLayout::CROSS_READ_SYSTEM_PROMPT_FILES {
            let path = project_dir.join(rel);
            let Some(content) = crate::common::read_cross_read_file(&path) else {
                continue;
            };
            diags.push(crate::common::cross_read_hint(rel, "another harness"));
            sources.push(path);
            crate::common::append_cross_read_to_system_prompt(&mut doc, &mut extracted, &content);
        }
    }

    // .github/instructions/*.instructions.md --> [instructions.rules]
    // ref: https://code.visualstudio.com/docs/copilot/customization/custom-instructions#_types-of-instruction-files
    let rules = import_rules_dir(project_dir)?;
    sources.extend(rules.sources);
    extracted.extend(rules.extracted);
    if let Some(rules_table) = rules.rules_table {
        theta_manifest::set_rules_section(&mut doc, rules_table);
    }

    // cross-read: .claude/rules/*.md -> additional [instructions.rules]
    // copilot reads these via chat.instructionsFilesLocations
    // ref: https://code.visualstudio.com/docs/copilot/customization/custom-instructions#_instructions-file-locations
    if opts.cross_read {
        let claude_rules_dir = project_dir.join(CopilotLayout::CROSS_READ_CLAUDE_RULES_DIR);
        if claude_rules_dir.is_dir() {
            let cross_rules = import_claude_rules_cross_read(&claude_rules_dir, project_dir)?;
            sources.extend(cross_rules.sources);
            extracted.extend(cross_rules.extracted);
            diags.extend(cross_rules.diags);
            if let Some(cross_table) = cross_rules.rules_table {
                theta_manifest::merge_rules(&mut doc, cross_table);
            }
        }
    }

    // .github/skills/*/SKILL.md --> [skills]
    // ref: https://code.visualstudio.com/docs/copilot/customization/agent-skills
    let skills = import_skills_block(project_dir)?;
    sources.extend(skills.sources);
    extracted.extend(skills.extracted);
    if let Some(t) = skills.skills_table {
        theta_manifest::set_section(&mut doc, "skills", t)?;
    }

    // .vscode/mcp.json --> [tools] (+ extras into [harness.github_copilot.tool.*])
    // ref: https://code.visualstudio.com/docs/copilot/reference/mcp-configuration
    let McpImportOutput {
        tools_table,
        extras_per_server: mcp_extras_per_server,
        input_variables: mcp_input_variables,
        sources: mcp_sources,
    } = import_mcp_file(project_dir)?;
    sources.extend(mcp_sources);
    if let Some(t) = tools_table {
        theta_manifest::set_section(&mut doc, "tools", t)?;
    }

    // .github/agents/*.agent.md --> [[subagents]] (+ extras)
    // ref: https://code.visualstudio.com/docs/copilot/customization/custom-agents
    let agents = import_agents_dir(project_dir, opts)?;
    sources.extend(agents.sources);
    if let Some(arr) = agents.subagents_array {
        theta_manifest::set_subagents(&mut doc, arr);
    }
    let subagent_extras = agents.extras_per_agent;

    // .github/prompts/ --> not modeled (hint only)
    if project_dir.join(CopilotLayout::prompts_dir()).is_dir() {
        diags.push(Diagnostic::hint(
            ".github/prompts/",
            "prompt files are a VS Code feature not modeled in theta - \
             if you need reusable command templates, consider using skills",
        ));
    }

    // .github/hooks/*.json --> [harness.github_copilot].hooks
    // theta imports the hook config (event --> command mapping) but does NOT
    // manage the scripts those commands reference.  if a command points to
    // a script that doesnt exist, that is the user problem (similar to MCP)
    // ref: https://code.visualstudio.com/docs/copilot/customization/hooks
    let (merged_hooks, hook_sources, hook_diags) = import_hooks_dir(project_dir)?;
    sources.extend(hook_sources);
    diags.extend(hook_diags);

    // .vscode/settings.json --> base [harness.github_copilot] table
    // ref: https://code.visualstudio.com/docs/copilot/reference/copilot-settings
    let (mut gp_table, settings_sources) = import_settings_file(project_dir)?;
    sources.extend(settings_sources);

    // overlay everything that lives under [harness.github_copilot] onto gp_table
    if !merged_hooks.is_empty() {
        gp_table["hooks"] = json_to_toml_item(&serde_json::Value::Object(merged_hooks));
    }
    extend_gp_table_with_extras(
        &mut gp_table,
        "tool",
        &mcp_extras_per_server,
        &mut diags,
        |name| format!("{} --> {name}", CopilotLayout::mcp().display()),
    );
    if let Some(ref inputs) = mcp_input_variables {
        let context = format!("{} --> inputs", CopilotLayout::mcp().display());
        gp_table["mcp_input_variables"] =
            json_to_toml_item_with_diagnostics(inputs, &context, &mut diags);
    }
    extend_gp_table_with_extras(
        &mut gp_table,
        "subagent",
        &subagent_extras,
        &mut diags,
        |name| format!(".github/agents/{name}.agent.md"),
    );

    attach_harness_table(&mut doc, gp_table)?;
    reorder_import_document(&mut doc);

    Ok(ImportResult {
        document: doc,
        extracted_files: extracted,
        sources_read: sources,
        diagnostics: diags,
    })
}

// helpers

/// Hang the `[harness.github_copilot]` partial table off `doc`, folding in the
/// detected harness version. No-op when the table and version are both empty.
fn attach_harness_table(
    doc: &mut toml_edit::DocumentMut,
    mut gp_table: toml_edit::Table,
) -> Result<()> {
    let has_version = crate::harnesses::version::detect_copilot_version();
    if gp_table.is_empty() && has_version.is_none() {
        return Ok(());
    }
    if let Some(ref ver) = has_version {
        gp_table["version"] = toml_edit::value(ver.clone());
    }
    let mut harness_table = toml_edit::Table::new();
    harness_table[theta_harness::HarnessTarget::Copilot.toml_key()] =
        toml_edit::Item::Table(gp_table);
    theta_manifest::set_section(doc, "harness", harness_table)?;
    Ok(())
}

/// Fold generic JSON extras into `gp_table[key]` as a nested table keyed by
/// `<inner_name>`. Each entry is routed through
/// `json_to_toml_item_with_diagnostics` with a caller-supplied context.
fn extend_gp_table_with_extras<F>(
    gp_table: &mut toml_edit::Table,
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
    gp_table[key] = toml_edit::Item::Table(table);
}

struct McpImportOutput {
    /// TOML table to install at `doc["tools"]`, or `None` when no servers block.
    tools_table: Option<toml_edit::Table>,
    /// Per-server non-typed keys for `[harness.github_copilot.tool.<name>]`.
    extras_per_server: BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    /// Top-level `inputs` array for `[harness.github_copilot.mcp_input_variables]`.
    input_variables: Option<serde_json::Value>,
    sources: Vec<PathBuf>,
}

/// Parse `.vscode/mcp.json` — a missing file is not an error.
///
/// ref: <https://code.visualstudio.com/docs/copilot/reference/mcp-configuration>
fn import_mcp_file(project_dir: &Path) -> Result<McpImportOutput> {
    let mut out = McpImportOutput {
        tools_table: None,
        extras_per_server: BTreeMap::new(),
        input_variables: None,
        sources: Vec::new(),
    };

    let mcp_path = project_dir.join(CopilotLayout::mcp());
    if !mcp_path.exists() {
        return Ok(out);
    }

    let raw = fs_err::read_to_string(&mcp_path)?;
    out.sources.push(CopilotLayout::mcp());

    let json = crate::common::parse_jsonc_value(&raw, &mcp_path)?;

    if let Some(servers) = json.get("servers").and_then(|v| v.as_object()) {
        let mut tools_table = toml_edit::Table::new();

        for (name, server) in servers {
            let mut tool = toml_edit::Table::new();

            let server_type = server
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("stdio");

            if server_type == "stdio" {
                if let Some(cmd) = server.get("command").and_then(|v| v.as_str()) {
                    let mut arr = toml_edit::Array::new();
                    arr.push(cmd);
                    tool["command"] = toml_edit::value(arr);
                }
                if let Some(args) = server.get("args").and_then(|v| v.as_array()) {
                    let arr: toml_edit::Array = args.iter().filter_map(|a| a.as_str()).collect();
                    if !arr.is_empty() {
                        tool["args"] = toml_edit::value(arr);
                    }
                }
            } else {
                // http or sse transport
                if let Some(url) = server.get("url").and_then(|v| v.as_str()) {
                    tool["url"] = toml_edit::value(url);
                }
                if let Some(headers) = server
                    .get("headers")
                    .and_then(|v| v.as_object())
                    .filter(|h| !h.is_empty())
                {
                    let mut hdr_table = toml_edit::InlineTable::new();
                    for (k, v) in headers {
                        if let Some(s) = v.as_str() {
                            hdr_table.insert(k, s.into());
                        }
                    }
                    tool["headers"] = toml_edit::value(hdr_table);
                }
            }

            if let Some(env) = server
                .get("env")
                .and_then(|v| v.as_object())
                .filter(|e| !e.is_empty())
            {
                let mut env_table = toml_edit::InlineTable::new();
                for (k, v) in env {
                    if let Some(s) = v.as_str() {
                        env_table.insert(k, s.into());
                    }
                }
                tool["env"] = toml_edit::value(env_table);
            }

            // generic extras: everything not in `theta_static::THETA_TYPED_MCP_KEYS`
            // lands in `[harness.github_copilot.tool.<name>]` on round-trip
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
    }

    // top-level `inputs` array --> `[harness.github_copilot.mcp_input_variables]`
    if let Some(inputs) = json.get("inputs") {
        out.input_variables = Some(inputs.clone());
    }

    Ok(out)
}

// hooks

/// Read `.github/hooks/*.json` and union event arrays into a single event map.
///
/// Returns `(merged, sources, diags)`. Accepts both the wrapped
/// `{"hooks": {...}}` and bare `{"<Event>": [...]}` shapes
///
/// ref: <https://code.visualstudio.com/docs/copilot/customization/hooks>
fn import_hooks_dir(project_dir: &Path) -> Result<HookImportOutput> {
    let mut merged: HookMap = HookMap::new();
    let mut sources: Vec<PathBuf> = Vec::new();
    let mut diags: Vec<Diagnostic> = Vec::new();

    let hooks_dir = project_dir.join(CopilotLayout::hooks_dir());
    if !hooks_dir.is_dir() {
        return Ok((merged, sources, diags));
    }

    let mut hook_files: Vec<PathBuf> = fs_err::read_dir(&hooks_dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    hook_files.sort();

    for hf in hook_files {
        let rel = hf.strip_prefix(project_dir).unwrap_or(&hf).to_path_buf();
        let raw = fs_err::read_to_string(&hf)?;
        let parsed: serde_json::Value = match crate::common::parse_jsonc_value(&raw, &hf) {
            Ok(v) => v,
            Err(e) => {
                diags.push(Diagnostic::warn(
                    rel.display().to_string(),
                    format!("failed to parse hooks file: {e}"),
                ));
                continue;
            }
        };
        sources.push(rel.clone());
        let hooks_obj = parsed
            .get("hooks")
            .and_then(|v| v.as_object())
            .cloned()
            .or_else(|| parsed.as_object().cloned());
        let Some(obj) = hooks_obj else {
            diags.push(Diagnostic::warn(
                rel.display().to_string(),
                "hooks file must be a JSON object (optionally wrapped in a \"hooks\" key)",
            ));
            continue;
        };
        for (event, value) in obj {
            match merged.get_mut(&event) {
                Some(existing) => match (existing, value) {
                    (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
                        a.extend(b);
                    }
                    (slot, incoming) => {
                        *slot = incoming;
                    }
                },
                None => {
                    merged.insert(event, value);
                }
            }
        }
    }

    Ok((merged, sources, diags))
}

// settings

/// Read `.vscode/settings.json` and project its keys into a partial
/// `[harness.github_copilot]` table - a missing file is not an error
///
/// ref: <https://code.visualstudio.com/docs/copilot/reference/copilot-settings>
fn import_settings_file(project_dir: &Path) -> Result<(toml_edit::Table, Vec<PathBuf>)> {
    let mut gp_table = toml_edit::Table::new();
    let mut sources: Vec<PathBuf> = Vec::new();

    let settings_path = project_dir.join(CopilotLayout::settings());
    if !settings_path.exists() {
        return Ok((gp_table, sources));
    }

    let raw = fs_err::read_to_string(&settings_path)?;
    sources.push(CopilotLayout::settings());

    let json = crate::common::parse_jsonc_map(&raw, &settings_path)?;

    for (key, value) in &json {
        if key.starts_with("github.copilot.") || key.starts_with("chat.") {
            gp_table[key.as_str()] = json_to_toml_item(value);
        }
    }

    Ok((gp_table, sources))
}

// system prompt + identity

/// Output of importing the identity header + system prompt body.
struct SystemPromptImport {
    agent_name: String,
    agent_description: String,
    /// System-prompt body written as an extracted file, if non-empty.
    extracted_file: Option<(PathBuf, String)>,
    sources: Vec<PathBuf>,
    diags: Vec<Diagnostic>,
}

/// Import `.github/copilot-instructions.md` as an opaque system prompt.
///
/// ref: <https://code.visualstudio.com/docs/copilot/customization/custom-instructions>
fn import_system_prompt(project_dir: &Path) -> Result<SystemPromptImport> {
    let mut out = SystemPromptImport {
        agent_name: default_agent_name(project_dir),
        agent_description: "imported from copilot".into(),
        extracted_file: None,
        sources: Vec::new(),
        diags: Vec::new(),
    };

    let sys_path = project_dir.join(CopilotLayout::system_prompt());
    if !sys_path.exists() {
        return Ok(out);
    }

    let content = fs_err::read_to_string(&sys_path)?;
    out.sources.push(CopilotLayout::system_prompt());

    if !content.trim().is_empty() {
        out.extracted_file = Some((PathBuf::from(theta_static::SYSTEM_FILE_NAME), content));
    }

    Ok(out)
}

// rules

/// Derive the `apply` / `apply_to` cell from a parsed rule frontmatter.
///
/// VS Code semantics:
///
///   `applyTo = "**"`              --> `apply = always`
///   `applyTo = "<glob>, ..."`     --> `apply = glob` + `apply_to` list
///   no `applyTo`, has description --> `apply = model-decision`
///   no `applyTo`, no description  --> `apply = always`
fn rule_apply_from_frontmatter(
    parsed: &crate::common::ParsedFrontmatter,
) -> (&'static str, Option<Vec<String>>) {
    let apply_to = parsed.get_str_list("applyTo").or_else(|| {
        parsed
            .get_str("applyTo")
            .map(|s| s.split(',').map(|p| p.trim().to_string()).collect())
    });

    match apply_to {
        Some(ref pats) if pats.len() == 1 && pats[0] == "**" => ("always", None),
        Some(pats) => ("glob", Some(pats)),
        None if parsed.get_str("description").is_some() => ("model-decision", None),
        None => ("manual", None),
    }
}

/// Output of importing the instructions directory.
struct RulesImport {
    rules_table: Option<toml_edit::Table>,
    extracted: Vec<crate::CastFile>,
    sources: Vec<PathBuf>,
}

/// Parse `.github/instructions/*.instructions.md` into a `[instructions.rules]`
/// table plus the extracted rule bodies.
///
/// ref: <https://code.visualstudio.com/docs/copilot/customization/custom-instructions#_types-of-instruction-files>
/// Recursively collect `*.instructions.md` files under a directory.
/// Returns `(stem, path)` pairs sorted by stem, where stem is the
/// path-qualified rule name relative to `base` (e.g. `review/pr-review`).
///
/// ref: VS Code searches `.github/instructions/` recursively.
/// <https://code.visualstudio.com/docs/copilot/customization/custom-instructions>
fn collect_instruction_files(dir: &Path, base: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = fs_err::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.filter_map(std::result::Result::ok).collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_instruction_files(&path, base, out);
        } else if let Some(fname) = path.file_name().and_then(|f| f.to_str()) {
            if fname.ends_with(".instructions.md") {
                let Some(rel) = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_str()
                    .map(String::from)
                else {
                    continue;
                };
                if let Some(stem) = rel.strip_suffix(".instructions.md") {
                    let stem = stem.trim_end_matches('/');
                    if !stem.is_empty() && stem.split('/').all(|s| !s.is_empty()) {
                        out.push((stem.to_string(), path));
                    }
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

    let instructions_dir = project_dir.join(".github/instructions");
    if !instructions_dir.is_dir() {
        return Ok(out);
    }

    let mut instruction_files = Vec::new();
    collect_instruction_files(&instructions_dir, &instructions_dir, &mut instruction_files);

    if instruction_files.is_empty() {
        return Ok(out);
    }

    let mut rules_table = toml_edit::Table::new();

    for (stem, path) in &instruction_files {
        let content = fs_err::read_to_string(path)?;
        out.sources.push(CopilotLayout::rule(stem));

        let parsed = parse_frontmatter(&content);

        let mut rule = toml_edit::Table::new();
        let rule_rel = theta_static::ThetaProjectLayout::rule_rel(stem);
        rule["src"] = toml_edit::value(rule_rel.as_str());

        if let Some(desc) = parsed.get_str("description") {
            rule["description"] = toml_edit::value(desc);
        }

        let (apply_mode, apply_to) = rule_apply_from_frontmatter(&parsed);
        rule["apply"] = toml_edit::value(apply_mode);
        if let Some(pats) = apply_to {
            let mut arr = toml_edit::Array::new();
            for p in &pats {
                arr.push(p.as_str());
            }
            rule["apply_to"] = toml_edit::value(arr);
        }

        rules_table[stem.as_str()] = toml_edit::Item::Table(rule);
        out.extracted
            .push((PathBuf::from(&rule_rel), parsed.content.into()));
    }

    out.rules_table = Some(rules_table);
    Ok(out)
}

struct CrossReadRulesImport {
    rules_table: Option<toml_edit::Table>,
    extracted: Vec<(PathBuf, crate::CastContent)>,
    sources: Vec<PathBuf>,
    diags: Vec<Diagnostic>,
}

/// Import `.claude/rules/*.md` as cross-read rules.
/// Claude rules use `paths` for globs (not `applyTo`).
fn import_claude_rules_cross_read(
    rules_dir: &Path,
    project_dir: &Path,
) -> Result<CrossReadRulesImport> {
    let mut out = CrossReadRulesImport {
        rules_table: None,
        extracted: Vec::new(),
        sources: Vec::new(),
        diags: Vec::new(),
    };

    let mut entries: Vec<_> = fs_err::read_dir(rules_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);

    if entries.is_empty() {
        return Ok(out);
    }

    let mut rules_table = toml_edit::Table::new();

    for entry in &entries {
        let fname = crate::common::filename_to_string(entry)?;
        let stem = fname.strip_suffix(".md").expect("filtered to .md");
        // prefix to avoid name collisions with native rules
        let key = format!("claude-{stem}");

        let content = fs_err::read_to_string(entry.path())?;
        let abs = entry.path();
        let rel_source = abs.strip_prefix(project_dir).unwrap_or(&abs);
        out.sources.push(rel_source.to_path_buf());

        let parsed = parse_frontmatter(&content);

        let rule_rel = theta_static::ThetaProjectLayout::rule_rel(&key);
        let mut rule = toml_edit::Table::new();
        rule["src"] = toml_edit::value(rule_rel.as_str());

        if let Some(desc) = parsed.get_str("description") {
            rule["description"] = toml_edit::value(desc);
        }

        // claude uses `paths` for globs (array of strings)
        if let Some(paths_str) = parsed.get_str("paths") {
            let globs: Vec<&str> = paths_str
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if globs.is_empty() {
                rule["apply"] = toml_edit::value("always");
            } else {
                rule["apply"] = toml_edit::value("glob");
                let mut arr = toml_edit::Array::new();
                for g in &globs {
                    arr.push(*g);
                }
                rule["apply_to"] = toml_edit::value(arr);
            }
        } else {
            rule["apply"] = toml_edit::value("always");
        }

        rules_table[key.as_str()] = toml_edit::Item::Table(rule);
        out.extracted
            .push((PathBuf::from(&rule_rel), parsed.content.into()));

        out.diags.push(Diagnostic::hint(
            format!("[cross-read] .claude/rules/{fname}"),
            format!(
                "imported .claude/rules/{fname} as rule \"{key}\" - \
                 this file is from the .claude/ directory and may duplicate on round-trip"
            ),
        ));
    }

    out.rules_table = Some(rules_table);
    Ok(out)
}

// skills

/// Output of importing the skills directory.
struct SkillsImport {
    skills_table: Option<toml_edit::Table>,
    extracted: Vec<crate::CastFile>,
    sources: Vec<PathBuf>,
}

/// Import `.github/skills/*/SKILL.md` into a `[skills]` table.
///
/// ref: <https://code.visualstudio.com/docs/copilot/customization/agent-skills>
fn import_skills_block(project_dir: &Path) -> Result<SkillsImport> {
    let mut out = SkillsImport {
        skills_table: None,
        extracted: Vec::new(),
        sources: Vec::new(),
    };

    let skill_results = import_skills_from_dir(project_dir, &CopilotLayout::skills_dir())?;
    if skill_results.is_empty() {
        return Ok(out);
    }

    let mut skills_table = toml_edit::Table::new();
    for (name, cast_files, source_rel) in skill_results {
        skills_table[&name] = toml_edit::Item::Table(theta_manifest::local_skill_entry(&name));
        out.extracted.extend(cast_files);
        out.sources.push(source_rel);
    }
    out.skills_table = Some(skills_table);
    Ok(out)
}

// subagents

/// Output of importing the `.github/agents/` directory.
struct AgentsImport {
    subagents_array: Option<toml_edit::ArrayOfTables>,
    /// Per-subagent non-typed frontmatter for `[harness.github_copilot.subagent.<name>]`.
    extras_per_agent: BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
    sources: Vec<PathBuf>,
}

/// Parse a single `.agent.md` / `.md` file under `.github/agents/` into a
/// `[[subagents]]` table plus the non-theta-typed frontmatter
///
/// VS Code also picks up plain `.md` files here, so both suffixes are accepted
fn import_one_agent_file(
    path: &Path,
    opts: &crate::ImportOptions,
    manifest_dir: &Path,
) -> Result<(
    String,
    toml_edit::Table,
    serde_json::Map<String, serde_json::Value>,
)> {
    let fname = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("path has no filename: {}", path.display()))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 filename: {}", path.display()))?
        .to_string();
    let stem = if let Some(s) = fname.strip_suffix(".agent.md") {
        s.to_string()
    } else {
        fname.strip_suffix(".md").unwrap_or(&fname).to_string()
    };

    let content = fs_err::read_to_string(path)?;
    let parsed = parse_frontmatter(&content);

    let mut table = toml_edit::Table::new();
    let agent_name = parsed.get_str("name").unwrap_or(&stem).to_string();
    table["name"] = toml_edit::value(&agent_name);

    let description = parsed
        .get_str("description")
        .unwrap_or("imported from .github/agents/")
        .to_string();
    table["description"] = toml_edit::value(description);

    let body = parsed.content.trim().to_string();
    if !body.is_empty() {
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

    // tools: VS Code uses a YAML array; also handle comma-separated strings
    if let Some(tools) = parsed.get_str_list("tools") {
        if !tools.is_empty() {
            let mut tools_arr = toml_edit::Array::new();
            for tool in &tools {
                tools_arr.push(tool.as_str());
            }
            table["tools"] = toml_edit::value(tools_arr);
        }
    }

    // preserve every non-theta-typed frontmatter field under
    // `[harness.github_copilot.subagent.<agent_name>]`
    let mut extras = serde_json::Map::new();
    for (k, v) in &parsed.data {
        if theta_static::THETA_TYPED_AGENT_KEYS.contains(&k.as_str()) {
            continue;
        }
        extras.insert(k.clone(), v.clone());
    }

    Ok((stem, table, extras))
}

/// Import `.github/agents/*.agent.md` (and plain `.md`) into `[[subagents]]`
/// plus per-agent extras.
///
/// ref: <https://code.visualstudio.com/docs/copilot/customization/custom-agents>
fn import_agents_dir(project_dir: &Path, opts: &crate::ImportOptions) -> Result<AgentsImport> {
    let mut out = AgentsImport {
        subagents_array: None,
        extras_per_agent: BTreeMap::new(),
        sources: Vec::new(),
    };

    let agents_dir = project_dir.join(CopilotLayout::agents_dir());
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
        let stem = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| {
                s.strip_suffix(".agent.md")
                    .or_else(|| s.strip_suffix(".md"))
                    .unwrap_or(s)
            })
            .unwrap_or_default()
            .to_string();
        out.sources.push(CopilotLayout::agent(&stem));

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
