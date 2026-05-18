use std::path::{Path, PathBuf};

use crate::ImportResult;
use crate::common::{
    CastFile, default_agent_name, import_skills_from_dir, json_to_toml_item_with_diagnostics,
    new_import_document, parse_frontmatter, reorder_import_document, set_import_agent,
    split_frontmatter, strip_identity_header_with_shape,
};
use anyhow::Result;
use theta_harness::layout::{CursorLayout, HarnessLayout};
use theta_schema::Diagnostic;

// .mdc frontmatter parser
/// Cursor .mdc frontmatter is NOT YAML - it's a simple `key: rest-of-line` format.
///
/// Cursor allows values that are invalid YAML:
/// - `globs: *.ts` - leading `*` is the YAML c-alias indicator
/// - `description: Go patterns: Repository` - unquoted `: ` in values
/// - `globs: "docs/**/*.md, docs/**/*.mdx"` - comma-separated glob string
///
/// ref: cursor.com/docs/rules#rule-anatomy (only 3 fields: description, globs, alwaysApply)
/// ref: github.com/nedcodes-ok/cursor-doctor - independent .mdc parser uses indexOf(':')
/// ref: github.com/nedcodes-ok/rule-porter - same line-based approach
pub(super) struct MdcFrontmatter {
    pub description: Option<String>,
    pub globs: Option<Vec<String>>,
    pub always_apply: bool,
    pub body: String,
}

pub(super) fn parse_mdc_frontmatter(input: &str) -> MdcFrontmatter {
    let (block, body) = split_frontmatter(input);
    let Some(block) = block else {
        return MdcFrontmatter {
            description: None,
            globs: None,
            always_apply: false,
            body: body.to_string(),
        };
    };

    let mut description = None;
    let mut globs = None;
    let mut always_apply = false;

    for line in block.lines() {
        let Some(colon) = line.find(':') else {
            continue;
        };
        let key = line[..colon].trim();
        let val = line[colon + 1..].trim();

        match key {
            super::MDC_DESCRIPTION => {
                if !val.is_empty() {
                    description = Some(val.to_string());
                }
            }
            super::MDC_GLOBS => {
                if !val.is_empty() {
                    globs = Some(parse_mdc_globs(val));
                }
            }
            super::MDC_ALWAYS_APPLY => {
                always_apply = val.eq_ignore_ascii_case("true");
            }
            _ => {} // unknown keys ignored
        }
    }

    MdcFrontmatter {
        description,
        globs,
        always_apply,
        body: body.to_string(),
    }
}

/// Parse the globs value from `.mdc` frontmatter.
///
/// Cursor supports two formats:
/// - JSON array: `["*.ts", "*.tsx"]` --> multiple patterns
/// - comma-separated string: `*.ts, *.tsx` or `"docs/**/*.md, docs/**/*.mdx"`
///
/// Cursor splits on commas (ref: cursor.com/docs/rules#glob-pattern-examples).
/// Brace expansion `{a,b}` is broken in Cursor itself — commas inside braces
/// are consumed by the splitter, and this parser matches that behavior.
pub(super) fn parse_mdc_globs(val: &str) -> Vec<String> {
    let val = val.trim();
    let val = val
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| val.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(val);

    if val.starts_with('[') {
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(val) {
            return arr;
        }
    }

    val.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

// import: system prompt
// ref: https://cursor.com/docs/rules
struct SystemPromptImport {
    agent_name: String,
    agent_description: String,
    extracted_file: Option<(PathBuf, String)>,
    sources: Vec<PathBuf>,
    diags: Vec<Diagnostic>,
}

fn import_system_prompt(project_dir: &Path) -> Result<SystemPromptImport> {
    let sys_path = project_dir.join(CursorLayout::system_prompt());
    if !sys_path.exists() {
        return Ok(SystemPromptImport {
            agent_name: default_agent_name(project_dir),
            agent_description: "imported from cursor".into(),
            extracted_file: None,
            sources: vec![],
            diags: vec![Diagnostic::hint(
                ".cursor/rules/system.md",
                "not found - using default agent name",
            )],
        });
    }

    let content = fs_err::read_to_string(&sys_path)?;
    let parsed = parse_frontmatter(&content);
    let (name, desc, system_body, _) = strip_identity_header_with_shape(&parsed.content);

    let extracted = if system_body.trim().is_empty() {
        None
    } else {
        Some((PathBuf::from(theta_static::SYSTEM_FILE_NAME), system_body))
    };

    Ok(SystemPromptImport {
        agent_name: name.unwrap_or_else(|| default_agent_name(project_dir)),
        agent_description: desc.unwrap_or_else(|| "imported from cursor".into()),
        extracted_file: extracted,
        sources: vec![CursorLayout::system_prompt()],
        diags: vec![],
    })
}

// import: rules

struct RulesImport {
    rules_table: Option<toml_edit::Table>,
    extracted: Vec<CastFile>,
    sources: Vec<PathBuf>,
}

/// .cursor/rules/*.{md,mdc} → [instructions.rules]
/// ref: <https://cursor.com/docs/rules>
fn import_rules(project_dir: &Path) -> Result<RulesImport> {
    let rules_dir = project_dir.join(".cursor/rules");
    if !rules_dir.is_dir() {
        return Ok(RulesImport {
            rules_table: None,
            extracted: vec![],
            sources: vec![],
        });
    }

    let mut entries: Vec<_> = fs_err::read_dir(&rules_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let p = e.path();
            let is_rule = p.extension().is_some_and(|x| x == "mdc" || x == "md");
            let is_system = e.file_name() == theta_static::SYSTEM_FILE_NAME;
            is_rule && !is_system
        })
        .collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);

    if entries.is_empty() {
        return Ok(RulesImport {
            rules_table: None,
            extracted: vec![],
            sources: vec![],
        });
    }

    let mut rules_table = toml_edit::Table::new();
    let mut extracted = Vec::new();
    let mut sources = Vec::new();

    for entry in entries {
        let fname = crate::common::filename_to_string(&entry)?;
        let stem = fname
            .strip_suffix(".mdc")
            .or_else(|| fname.strip_suffix(".md"))
            .expect("filtered to .md/.mdc");

        let content = fs_err::read_to_string(entry.path())?;
        sources.push(PathBuf::from(format!(".cursor/rules/{fname}")));

        let mdc = parse_mdc_frontmatter(&content);

        let mut rule = toml_edit::Table::new();
        let rule_rel = theta_static::ThetaProjectLayout::rule_rel(stem);
        rule["src"] = toml_edit::value(rule_rel.as_str());

        if let Some(ref desc) = mdc.description {
            rule["description"] = toml_edit::value(desc.as_str());
        }

        // ref: https://cursor.com/docs/rules#rule-anatomy
        if mdc.always_apply {
            rule["apply"] = toml_edit::value("always");
            if let Some(ref pats) = mdc.globs {
                rule["apply_to"] = toml_edit::value(globs_to_toml_array(pats));
            }
        } else if let Some(ref pats) = mdc.globs {
            rule["apply"] = toml_edit::value("glob");
            rule["apply_to"] = toml_edit::value(globs_to_toml_array(pats));
        } else {
            rule["apply"] = toml_edit::value("model-decision");
        }

        rules_table[stem] = toml_edit::Item::Table(rule);
        extracted.push((PathBuf::from(&rule_rel), mdc.body.into()));
    }

    Ok(RulesImport {
        rules_table: Some(rules_table),
        extracted,
        sources,
    })
}

fn globs_to_toml_array(pats: &[String]) -> toml_edit::Array {
    let mut arr = toml_edit::Array::new();
    for p in pats {
        arr.push(p.as_str());
    }
    arr
}

// import: skills

struct SkillsImport {
    skills_table: Option<toml_edit::Table>,
    extracted: Vec<CastFile>,
    sources: Vec<PathBuf>,
    diags: Vec<Diagnostic>,
}

/// .cursor/skills/ + .agents/skills/ → [skills]
/// ref: <https://cursor.com/docs/skills>
fn import_skills(project_dir: &Path) -> Result<SkillsImport> {
    let native = import_skills_from_dir(project_dir, &CursorLayout::skills_dir())?;
    let cross = import_skills_from_dir(project_dir, &CursorLayout::cross_agent_skills_dir())?;

    if native.is_empty() && cross.is_empty() {
        return Ok(SkillsImport {
            skills_table: None,
            extracted: vec![],
            sources: vec![],
            diags: vec![],
        });
    }

    let mut skills_table = toml_edit::Table::new();
    let mut extracted = Vec::new();
    let mut sources = Vec::new();
    let mut diags = Vec::new();
    let mut imported_names = std::collections::BTreeSet::new();

    for (name, cast_files, source_rel) in native {
        skills_table[&name] = toml_edit::Item::Table(theta_manifest::local_skill_entry(&name));
        extracted.extend(cast_files);
        sources.push(source_rel);
        imported_names.insert(name);
    }

    for (name, cast_files, source_rel) in cross {
        if imported_names.contains(&name) {
            diags.push(Diagnostic::hint(
                format!(".agents/skills/{name}/SKILL.md"),
                format!("skill \"{name}\" skipped - already imported from .cursor/skills/ (native takes precedence)"),
            ));
        } else {
            skills_table[&name] = toml_edit::Item::Table(theta_manifest::local_skill_entry(&name));
            extracted.extend(cast_files);
            sources.push(source_rel);
        }
    }

    Ok(SkillsImport {
        skills_table: if skills_table.is_empty() {
            None
        } else {
            Some(skills_table)
        },
        extracted,
        sources,
        diags,
    })
}

// import: MCP tools
struct McpImport {
    tools_table: Option<toml_edit::Table>,
    cursor_extras: toml_edit::Table,
    sources: Vec<PathBuf>,
    diags: Vec<Diagnostic>,
}

/// .cursor/mcp.json → [tools] + harness extras
/// ref: <https://cursor.com/docs/mcp>
fn import_mcp(project_dir: &Path) -> Result<McpImport> {
    let mcp_path = project_dir.join(CursorLayout::mcp());
    if !mcp_path.exists() {
        return Ok(McpImport {
            tools_table: None,
            cursor_extras: toml_edit::Table::new(),
            sources: vec![],
            diags: vec![],
        });
    }

    let raw = fs_err::read_to_string(&mcp_path)?;
    let json = crate::common::parse_jsonc_value(&raw, &mcp_path)?;
    let mut diags = Vec::new();
    let mut cursor_extras = toml_edit::Table::new();

    if json.get("mcpServers").is_none() {
        diags.push(Diagnostic::warn(
            ".cursor/mcp.json",
            "no \"mcpServers\" key found - file has no effect. Cursor expects {\"mcpServers\": {...}}",
        ));
        return Ok(McpImport {
            tools_table: None,
            cursor_extras,
            sources: vec![CursorLayout::mcp()],
            diags,
        });
    }

    let Some(servers) = json.get("mcpServers").and_then(|v| v.as_object()) else {
        return Ok(McpImport {
            tools_table: None,
            cursor_extras,
            sources: vec![CursorLayout::mcp()],
            diags,
        });
    };

    let mut tools_table = toml_edit::Table::new();

    for (name, server) in servers {
        let mut tool = toml_edit::Table::new();

        if let Some(cmd) = server.get("command").and_then(|v| v.as_str()) {
            tool["command"] = toml_edit::value(toml_edit::Array::from_iter([cmd]));
        }
        if let Some(url) = server.get("url").and_then(|v| v.as_str()) {
            tool["url"] = toml_edit::value(url);
        }
        if let Some(args) = server.get("args").and_then(|v| v.as_array()) {
            let arr: toml_edit::Array = args.iter().filter_map(|a| a.as_str()).collect();
            tool["args"] = toml_edit::value(arr);
        }
        if let Some(env) = server.get("env").and_then(|v| v.as_object()) {
            let mut tbl = toml_edit::InlineTable::new();
            for (k, v) in env {
                if let Some(s) = v.as_str() {
                    tbl.insert(k, s.into());
                } else if let Some(n) = v.as_i64() {
                    tbl.insert(k, n.to_string().into());
                } else if let Some(n) = v.as_f64() {
                    tbl.insert(k, n.to_string().into());
                }
            }
            tool["env"] = toml_edit::value(tbl);
        }
        if let Some(headers) = server.get("headers").and_then(|v| v.as_object()) {
            let mut tbl = toml_edit::InlineTable::new();
            for (k, v) in headers {
                if let Some(s) = v.as_str() {
                    tbl.insert(k, s.into());
                }
            }
            tool["headers"] = toml_edit::value(tbl);
        }

        let modeled_keys = ["command", "args", "url", "env", "headers"];
        let mut has_extras = false;
        for (key, _val) in server.as_object().into_iter().flatten() {
            if !modeled_keys.contains(&key.as_str()) {
                has_extras = true;
                diags.push(Diagnostic::hint(
                    format!(".cursor/mcp.json --> {name}.{key}"),
                    "field stored in [harness.cursor] extras (not modeled in [tools])",
                ));
            }
        }
        if has_extras {
            let extras_ctx = format!("harness.cursor.mcp_extras.{name}");
            let extras_item = json_to_toml_item_with_diagnostics(server, &extras_ctx, &mut diags);
            if !cursor_extras.contains_key("mcp_extras") {
                cursor_extras["mcp_extras"] = toml_edit::Item::Table(toml_edit::Table::new());
            }
            cursor_extras["mcp_extras"][name.as_str()] = extras_item;
        }

        tools_table[name] = toml_edit::Item::Table(tool);
    }

    Ok(McpImport {
        tools_table: if tools_table.is_empty() {
            None
        } else {
            Some(tools_table)
        },
        cursor_extras,
        sources: vec![CursorLayout::mcp()],
        diags,
    })
}

// import: hooks
struct HooksImport {
    hooks_item: Option<toml_edit::Item>,
    sources: Vec<PathBuf>,
    diags: Vec<Diagnostic>,
}

/// .cursor/hooks.json → [harness.cursor].hooks
/// ref: <https://cursor.com/docs/hooks>
fn import_hooks(project_dir: &Path) -> Result<HooksImport> {
    let hooks_path = project_dir.join(CursorLayout::hooks());
    if !hooks_path.exists() {
        return Ok(HooksImport {
            hooks_item: None,
            sources: vec![],
            diags: vec![],
        });
    }

    let raw = fs_err::read_to_string(&hooks_path)?;
    let json = crate::common::parse_jsonc_value(&raw, &hooks_path)?;
    let mut diags = Vec::new();

    if json.get("hooks").is_none() {
        diags.push(Diagnostic::warn(
            ".cursor/hooks.json",
            "no \"hooks\" key found - file has no effect. Cursor expects {\"version\": 1, \"hooks\": {...}}",
        ));
    }

    if let Some(hooks_obj) = json.get("hooks").and_then(|h| h.as_object()) {
        for (event, scripts) in hooks_obj {
            let Some(arr) = scripts.as_array() else {
                continue;
            };
            for (i, script) in arr.iter().enumerate() {
                if script
                    .get("loop_limit")
                    .is_some_and(serde_json::Value::is_null)
                {
                    diags.push(Diagnostic::warn(
                        format!(".cursor/hooks.json --> hooks.{event}[{i}].loop_limit"),
                        "loop_limit: null means unlimited in Cursor, but TOML has no null - key will be omitted (defaults to 5). set an explicit high number instead",
                    ));
                }
            }
        }
    }

    let ctx = format!(
        "harness.{}.hooks",
        theta_harness::HarnessTarget::Cursor.toml_key()
    );
    let item = json_to_toml_item_with_diagnostics(&json, &ctx, &mut diags);

    Ok(HooksImport {
        hooks_item: Some(item),
        sources: vec![CursorLayout::hooks()],
        diags,
    })
}

// import: agents (subagents)
struct AgentsImport {
    subagents: Option<toml_edit::ArrayOfTables>,
    extracted: Vec<CastFile>,
    sources: Vec<PathBuf>,
    cursor_subagent_extras: toml_edit::Table,
}

/// .cursor/agents/*.md → [[subagents]]
/// ref: <https://cursor.com/docs/subagents>
fn import_agents(project_dir: &Path) -> Result<AgentsImport> {
    let agents_dir = project_dir.join(CursorLayout::agents_dir());
    if !agents_dir.is_dir() {
        return Ok(AgentsImport {
            subagents: None,
            extracted: vec![],
            sources: vec![],
            cursor_subagent_extras: toml_edit::Table::new(),
        });
    }

    let mut entries: Vec<_> = fs_err::read_dir(&agents_dir)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);

    if entries.is_empty() {
        return Ok(AgentsImport {
            subagents: None,
            extracted: vec![],
            sources: vec![],
            cursor_subagent_extras: toml_edit::Table::new(),
        });
    }

    let mut subagents_arr = toml_edit::ArrayOfTables::new();
    let mut extracted = Vec::new();
    let mut sources = Vec::new();
    let mut cursor_subagent_extras = toml_edit::Table::new();

    for entry in entries {
        let fname = crate::common::filename_to_string(&entry)?;
        let name = fname
            .strip_suffix(".md")
            .expect("filtered to .md")
            .to_string();
        let content = fs_err::read_to_string(entry.path())?;
        sources.push(CursorLayout::agent(&name));

        // agents use valid YAML frontmatter (description, model, readonly, is_background)
        let parsed = parse_frontmatter(&content);
        let mut sub = toml_edit::Table::new();
        sub["name"] = toml_edit::value(name.as_str());
        let default_desc = format!("imported from .cursor/agents/{fname}");
        let desc = parsed.get_str("description").unwrap_or(&default_desc);
        sub["description"] = toml_edit::value(desc);
        if let Some(model) = parsed.get_str("model") {
            sub["model"] = toml_edit::value(model);
        }
        // cursor-specific fields go to [harness.cursor.subagent.<name>]
        let mut agent_extras = toml_edit::InlineTable::new();
        if let Some(readonly) = parsed.get_bool("readonly") {
            agent_extras.insert("readonly", toml_edit::Value::from(readonly));
        }
        if let Some(is_background) = parsed.get_bool("is_background") {
            agent_extras.insert("is_background", toml_edit::Value::from(is_background));
        }
        if !agent_extras.is_empty() {
            cursor_subagent_extras[&name] = toml_edit::value(agent_extras);
        }
        let body = parsed.content.trim();
        if !body.is_empty() {
            let rel = theta_static::ThetaProjectLayout::subagent_prompt_rel(&name);
            extracted.push((PathBuf::from(&rel), body.to_string().into()));
            sub["prompt_path"] = toml_edit::value(&rel);
        }
        subagents_arr.push(sub);
    }

    Ok(AgentsImport {
        subagents: if subagents_arr.is_empty() {
            None
        } else {
            Some(subagents_arr)
        },
        extracted,
        sources,
        cursor_subagent_extras,
    })
}

fn has_subagent_named(doc: &toml_edit::DocumentMut, name: &str) -> bool {
    doc.get("subagents")
        .and_then(|s| s.as_array_of_tables())
        .is_some_and(|arr| {
            arr.iter()
                .any(|t| t.get("name").and_then(|n| n.as_str()) == Some(name))
        })
}

fn append_subagent(doc: &mut toml_edit::DocumentMut, sub: toml_edit::Table) {
    theta_manifest::append_subagent(doc, sub);
}

/// Cross-read: import subagents from `.claude/agents/` and `.codex/agents/`.
/// ref: <https://cursor.com/docs/subagents#file-locations>
fn cross_read_agents(
    project_dir: &Path,
    doc: &mut toml_edit::DocumentMut,
    extracted: &mut Vec<crate::CastFile>,
    sources: &mut Vec<PathBuf>,
    diags: &mut Vec<Diagnostic>,
) -> Result<()> {
    for (dir_rel, ext) in CursorLayout::CROSS_READ_AGENT_DIRS {
        let dir = project_dir.join(dir_rel);
        if !dir.is_dir() {
            continue;
        }
        let mut entries: Vec<_> = fs_err::read_dir(&dir)?
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|e| e == *ext))
            .collect();
        entries.sort_by_key(fs_err::DirEntry::file_name);

        let suffix = format!(".{ext}");
        for entry in entries {
            let fname = crate::common::filename_to_string(&entry)?;
            let stem = fname.strip_suffix(&suffix).expect("filtered by extension");
            let label = format!("{dir_rel}/{fname}");

            if has_subagent_named(doc, stem) {
                diags.push(Diagnostic::hint(
                    format!("[cross-read] {label}"),
                    format!("skipped - native .cursor/agents/{stem}.md takes precedence"),
                ));
                continue;
            }

            let content = fs_err::read_to_string(entry.path())?;
            let abs = entry.path();
            sources.push(abs.strip_prefix(project_dir).unwrap_or(&abs).to_path_buf());

            let result = if *ext == "md" {
                let (sub, file) = crate::common::import_md_agent_as_subagent(stem, &content);
                Ok((sub, file))
            } else {
                crate::common::import_toml_agent_as_subagent(stem, &content, &label)
            };

            let Ok((sub, prompt_file)) = result else {
                diags.push(Diagnostic::warn(
                    format!("[cross-read] {label}"),
                    format!("failed to parse {label} - skipped"),
                ));
                continue;
            };

            if let Some(file) = prompt_file {
                extracted.push(file);
            }
            append_subagent(doc, sub);
            diags.push(crate::common::cross_read_hint(&label, "another harness"));
        }
    }
    Ok(())
}

// orchestrator

pub(super) fn import(project_dir: &Path, opts: &crate::ImportOptions) -> Result<ImportResult> {
    let mut doc = new_import_document();
    let mut extracted: Vec<crate::CastFile> = Vec::new();
    let mut sources = Vec::new();
    let mut diags = Vec::new();

    // .cursor/rules/system.md → identity + system prompt
    let sys = import_system_prompt(project_dir)?;
    sources.extend(sys.sources);
    diags.extend(sys.diags);
    set_import_agent(&mut doc, &sys.agent_name, &sys.agent_description);
    if let Some((path, body)) = sys.extracted_file {
        extracted.push((path, body.into()));
        theta_manifest::set_system_path(&mut doc);
    }

    // cross-read: AGENTS.md -> concatenate into system prompt
    // cursor treats AGENTS.md as always-on instructions (same as rules with alwaysApply: true)
    // ref: https://cursor.com/docs/rules#agentsmd
    if opts.cross_read {
        let agents_md = project_dir.join(CursorLayout::CROSS_READ_AGENTS_MD);
        if let Some(content) = crate::common::read_cross_read_file(&agents_md) {
            diags.push(crate::common::cross_read_hint(
                CursorLayout::CROSS_READ_AGENTS_MD,
                "another harness",
            ));
            sources.push(agents_md);
            crate::common::append_cross_read_to_system_prompt(&mut doc, &mut extracted, &content);
        }
    }

    // .cursor/rules/*.{md,mdc} → [instructions.rules]
    let rules = import_rules(project_dir)?;
    sources.extend(rules.sources);
    extracted.extend(rules.extracted);
    if let Some(rules_table) = rules.rules_table {
        theta_manifest::set_rules_section(&mut doc, rules_table);
    }

    // .cursor/skills/ + .agents/skills/ → [skills]
    let skills = import_skills(project_dir)?;
    sources.extend(skills.sources);
    extracted.extend(skills.extracted);
    diags.extend(skills.diags);
    if let Some(t) = skills.skills_table {
        theta_manifest::set_section(&mut doc, "skills", t)?;
    }

    // .cursor/mcp.json → [tools] + harness extras
    let mcp = import_mcp(project_dir)?;
    sources.extend(mcp.sources);
    diags.extend(mcp.diags);
    if let Some(t) = mcp.tools_table {
        theta_manifest::set_section(&mut doc, "tools", t)?;
    }

    // .cursor/hooks.json → [harness.cursor].hooks
    let hooks = import_hooks(project_dir)?;
    sources.extend(hooks.sources);
    diags.extend(hooks.diags);

    // .cursor/agents/*.md → [[subagents]]
    let agents = import_agents(project_dir)?;
    sources.extend(agents.sources);
    extracted.extend(agents.extracted);
    if let Some(arr) = agents.subagents {
        theta_manifest::set_subagents(&mut doc, arr);
    }

    if opts.cross_read {
        cross_read_agents(
            project_dir,
            &mut doc,
            &mut extracted,
            &mut sources,
            &mut diags,
        )?;
    }

    // assemble [harness.cursor]
    let mut cursor_table = mcp.cursor_extras;
    if let Some(item) = hooks.hooks_item {
        cursor_table["hooks"] = item;
    }
    if !agents.cursor_subagent_extras.is_empty() {
        cursor_table["subagent"] = toml_edit::Item::Table(agents.cursor_subagent_extras);
    }
    if let Some(ver) = crate::harnesses::version::detect_cursor_version() {
        cursor_table["version"] = toml_edit::value(ver);
    }
    if !cursor_table.is_empty() {
        let mut harness_table = toml_edit::Table::new();
        harness_table[theta_harness::HarnessTarget::Cursor.toml_key()] =
            toml_edit::Item::Table(cursor_table);
        theta_manifest::set_section(&mut doc, "harness", harness_table)?;
    }

    reorder_import_document(&mut doc);

    Ok(ImportResult {
        document: doc,
        extracted_files: extracted,
        sources_read: sources,
        diagnostics: diags,
    })
}
