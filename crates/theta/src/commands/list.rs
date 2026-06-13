//! `theta list` — list rules, tools, skills, or subagents.

#![allow(clippy::print_stdout)]

use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{ListCommand, ListNamespace, OutputFormat};
use theta_manifest::read_manifest;
use theta_schema::{Rule, ThetaManifest};
use theta_static::{StoreEntry, StoreIndexRuleEntry};
use theta_store::StoreHandle;

use super::output::present;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ListKind {
    Rules,
    Tools,
    Skills,
    Subagents,
    Store,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct ListOutput {
    pub kind: ListKind,
    pub entries: serde_json::Value,
}

pub(crate) fn execute(
    ns: ListNamespace,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    // store listing does not need a manifest
    if let ListCommand::Store = ns.command {
        return list_store(output_format);
    }

    super::require_manifest(manifest_path)?;

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let (kind, entries) = match ns.command {
        ListCommand::Rules => (
            ListKind::Rules,
            manifest
                .instructions
                .as_ref()
                .and_then(|i| i.rules.as_ref())
                .map_or_else(|| Ok(serde_json::json!({})), serde_json::to_value)?,
        ),
        ListCommand::Tools => (
            ListKind::Tools,
            manifest
                .tools
                .as_ref()
                .map_or_else(|| Ok(serde_json::json!({})), serde_json::to_value)?,
        ),
        ListCommand::Skills => (
            ListKind::Skills,
            manifest
                .skills
                .as_ref()
                .map_or_else(|| Ok(serde_json::json!({})), serde_json::to_value)?,
        ),
        ListCommand::Subagents => (
            ListKind::Subagents,
            manifest
                .subagents
                .as_ref()
                .map_or_else(|| Ok(serde_json::json!([])), serde_json::to_value)?,
        ),
        ListCommand::Store => unreachable!("handled above"),
    };

    let outcome = ListOutput { kind, entries };
    let manifest_for_render = manifest;
    present(
        &["list", kind_verb(&outcome.kind)],
        output_format,
        outcome,
        vec![],
        move |o| match o.kind {
            ListKind::Rules => render_rules(&manifest_for_render),
            ListKind::Tools => render_tools(&manifest_for_render),
            ListKind::Skills => render_skills(&manifest_for_render),
            ListKind::Subagents => render_subagents(&manifest_for_render),
            ListKind::Store => unreachable!(),
        },
    )
}

fn kind_verb(kind: &ListKind) -> &'static str {
    match kind {
        ListKind::Rules => "rules",
        ListKind::Tools => "tools",
        ListKind::Skills => "skills",
        ListKind::Subagents => "subagents",
        ListKind::Store => "store",
    }
}

fn list_store(output_format: OutputFormat) -> Result<()> {
    let store = StoreHandle::open()?;
    let index = store.load_index()?;

    let to_entry = |e: &StoreEntry| {
        serde_json::json!({
            "registered": e.registered,
            "source_project": e.source_project,
            "description": e.description,
        })
    };
    let to_rule_entry = |e: &StoreIndexRuleEntry| {
        serde_json::json!({
            "registered": e.registered,
            "source_project": e.source_project,
            "description": e.description,
            "apply": e.apply,
        })
    };

    let agents: serde_json::Map<String, serde_json::Value> = index
        .agents
        .iter()
        .map(|(k, v)| (k.clone(), to_entry(v)))
        .collect();
    let skills: serde_json::Map<String, serde_json::Value> = index
        .skills
        .iter()
        .map(|(k, v)| (k.clone(), to_entry(v)))
        .collect();
    let rules: serde_json::Map<String, serde_json::Value> = index
        .rules
        .iter()
        .map(|(k, v)| (k.clone(), to_rule_entry(v)))
        .collect();

    let outcome = ListOutput {
        kind: ListKind::Store,
        entries: serde_json::json!({
            "agents": agents,
            "skills": skills,
            "rules": rules,
        }),
    };
    let index_for_render = index;
    present(
        &["list", "store"],
        output_format,
        outcome,
        vec![],
        move |_| render_store(&index_for_render),
    )
}

fn render_rules(manifest: &ThetaManifest) {
    let rules = match manifest
        .instructions
        .as_ref()
        .and_then(|i| i.rules.as_ref())
    {
        Some(r) if !r.is_empty() => r,
        _ => {
            anstream::eprintln!("{} no rules registered", "info".blue().bold());
            return;
        }
    };

    println!(
        "  {} {} {} {}",
        "NAME".white().bold(),
        "APPLY".white().bold(),
        "SOURCE".white().bold(),
        "SUMMARY".white().bold(),
    );
    for (name, rule) in rules {
        println!("{}", format_rule_line(name, rule));
    }
}

fn render_tools(manifest: &ThetaManifest) {
    let tools = match manifest.tools.as_ref() {
        Some(t) if !t.is_empty() => t,
        _ => {
            anstream::eprintln!("{} no tools registered", "info".blue().bold());
            return;
        }
    };

    println!(
        "  {} {} {} {}",
        "NAME".white().bold(),
        "TYPE".white().bold(),
        "TARGET".white().bold(),
        "STATUS".white().bold(),
    );
    for (name, tool) in tools {
        let enabled = if tool.enabled { "" } else { " (disabled)" };
        println!(
            "  {} {} {} {}",
            name.cyan().bold(),
            tool.transport().dimmed(),
            tool.target(),
            enabled.yellow(),
        );
    }
}

fn render_skills(manifest: &ThetaManifest) {
    let skills = match manifest.skills.as_ref() {
        Some(s) if !s.is_empty() => s,
        _ => {
            anstream::eprintln!("{} no skills registered", "info".blue().bold());
            return;
        }
    };

    for (name, skill) in skills {
        let (source_type, source_ref) = skill.source.display_compact();
        println!(
            "  {} {} {}",
            name.cyan().bold(),
            source_type.dimmed(),
            source_ref,
        );
    }
}

fn render_subagents(manifest: &ThetaManifest) {
    let subs = match manifest.subagents.as_ref() {
        Some(s) if !s.is_empty() => s,
        _ => {
            anstream::eprintln!("{} no subagents registered", "info".blue().bold());
            return;
        }
    };

    for sub in subs {
        println!(
            "  {} {} {}",
            sub.name.cyan().bold(),
            sub.mode().dimmed(),
            sub.description.as_deref().unwrap_or(""),
        );
        if let Some(ref model) = sub.model {
            println!("    model: {}", model.dimmed());
        }
    }
}

// format a single rule line for display (shared by list and describe)
pub(crate) fn format_rule_line(name: &str, rule: &Rule) -> String {
    let apply = rule.apply.as_str();
    let src = rule.src.display_compact();
    match rule.summary.as_deref() {
        Some(summary) => format!(
            "  {} {} {} {}",
            name.cyan().bold(),
            apply.dimmed(),
            src,
            summary.dimmed(),
        ),
        None => format!("  {} {} {}", name.cyan().bold(), apply.dimmed(), src),
    }
}

fn render_store(index: &theta_static::StoreIndex) {
    let mut any = false;

    if !index.agents.is_empty() {
        any = true;
        println!("\n  {}", "AGENTS".white().bold());
        for (name, entry) in &index.agents {
            println!("    {}  {}", name.cyan().bold(), entry.description.dimmed());
        }
    }

    if !index.skills.is_empty() {
        any = true;
        println!("\n  {}", "SKILLS".white().bold());
        for (name, entry) in &index.skills {
            println!("    {}  {}", name.cyan().bold(), entry.description.dimmed());
        }
    }

    if !index.rules.is_empty() {
        any = true;
        println!("\n  {}", "RULES".white().bold());
        for (name, entry) in &index.rules {
            println!("    {}  {}", name.cyan().bold(), entry.description.dimmed());
        }
    }

    if !any {
        anstream::eprintln!("{} system store is empty", "info".blue().bold());
    }
}
