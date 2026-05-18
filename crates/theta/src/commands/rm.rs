//! `theta rm` — remove rules, tools, skills, or subagents from the manifest.

use std::path::Path;

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_cli::{
    RmCommand, RmNamespace, RmRuleArgs, RmSkillArgs, RmStoreArgs, RmSubagentArgs, RmSystemArgs,
    RmToolArgs,
};
use theta_manifest::{read_document, read_manifest, write_document};
use theta_static::is_default_manifest;

use super::{project_dir, require_manifest};

pub(crate) fn dispatch(ns: RmNamespace, manifest_path: &Path) -> Result<()> {
    match ns.command {
        RmCommand::Rule(args) => rm_rule(args, manifest_path),
        RmCommand::System(args) => rm_system(args, manifest_path),
        RmCommand::Tool(args) => rm_tool(args, manifest_path),
        RmCommand::Skill(args) => rm_skill(args, manifest_path),
        RmCommand::Subagent(args) => rm_subagent(args, manifest_path),
        RmCommand::Store(args) => rm_store(args),
    }
}

/// Attempt to delete source; print "removed {label} and deleted {src}" if successful.
/// Returns `Ok(true)` when the source was deleted (caller should `return Ok(())`).
fn try_delete_source(
    project_dir: &Path,
    source_path: Option<&str>,
    label: &str,
    is_dir: bool,
) -> Result<bool> {
    let Some(src) = source_path else {
        return Ok(false);
    };
    let full_path = project_dir.join(src);
    if !full_path.exists() {
        return Ok(false);
    }
    if is_dir {
        fs_err::remove_dir_all(&full_path)
    } else {
        fs_err::remove_file(&full_path)
    }
    .with_context(|| format!("failed to delete {}", full_path.display()))?;
    anstream::eprintln!(
        "{} {} and deleted {}",
        "removed".red().bold(),
        label,
        src.dimmed(),
    );
    Ok(true)
}

fn print_removed(label: &str) {
    anstream::eprintln!("{} {}", "removed".red().bold(), label);
}

fn rm_rule(args: RmRuleArgs, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;

    let project_dir = project_dir(manifest_path)?;

    // typed read to extract the source path before mutation
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    let source_path = manifest
        .instructions
        .as_ref()
        .and_then(|i| i.rules.as_ref())
        .and_then(|r| r.get(&args.name))
        .and_then(|rule| rule.src.local_path().map(std::string::ToString::to_string));

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let Some(instructions) = doc.get_mut("instructions").and_then(|i| i.as_table_mut()) else {
        bail!(
            "rule \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        )
    };

    let Some(rules) = instructions.get_mut("rules").and_then(|r| r.as_table_mut()) else {
        bail!(
            "rule \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        )
    };

    if !rules.contains_key(&args.name) {
        bail!(
            "rule \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        );
    }

    rules.remove(&args.name);

    if rules.is_empty() {
        instructions.remove("rules");
    }

    let instructions_empty = doc
        .get("instructions")
        .and_then(|i| i.as_table())
        .is_some_and(toml_edit::Table::is_empty);
    if instructions_empty {
        doc.as_table_mut().remove("instructions");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let label = format!("rule \"{}\"", args.name.cyan());
    if args.delete {
        try_delete_source(project_dir, source_path.as_deref(), &label, false)?;
    }

    print_removed(&label);
    if !args.delete {
        if let Some(ref src) = source_path {
            anstream::eprintln!(
                "{} source file {} was not deleted - use --delete to remove it",
                "hint".blue().bold(),
                src.dimmed(),
            );
        }
    }

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(theta_cli::SyncArgs { force: true }, manifest_path)?;
    }
    Ok(())
}

fn rm_system(args: RmSystemArgs, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;

    let project_dir = project_dir(manifest_path)?;
    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let source_path = doc
        .get("instructions")
        .and_then(|i| i.get("system"))
        .and_then(|s| s.as_str())
        .map(std::string::ToString::to_string);

    if source_path.is_none() {
        bail!(
            "[instructions].system is not set in {}",
            manifest_path.display()
        );
    }

    let instructions = doc["instructions"]
        .as_table_mut()
        .context("[instructions] is not a table")?;
    instructions.remove("system");

    if instructions.is_empty() {
        doc.as_table_mut().remove("instructions");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let label = "system prompt";
    if args.delete {
        try_delete_source(project_dir, source_path.as_deref(), label, false)?;
    }

    print_removed(label);

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(theta_cli::SyncArgs { force: true }, manifest_path)?;
    }
    Ok(())
}

fn rm_tool(args: RmToolArgs, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let Some(tools) = doc.get_mut("tools").and_then(|t| t.as_table_mut()) else {
        bail!(
            "tool \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        )
    };

    if !tools.contains_key(&args.name) {
        bail!(
            "tool \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        );
    }

    tools.remove(&args.name);

    if tools.is_empty() {
        doc.as_table_mut().remove("tools");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    print_removed(&format!("tool \"{}\"", args.name.cyan()));
    Ok(())
}

fn rm_skill(args: RmSkillArgs, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;

    let project_dir = project_dir(manifest_path)?;

    // typed read to extract the local path before mutation
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    let source_path = manifest
        .skills
        .as_ref()
        .and_then(|s| s.get(&args.name))
        .and_then(|skill| match &skill.source {
            theta_schema::SourceRef::Path { path } => Some(path.clone()),
            _ => None,
        });

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let Some(skills) = doc.get_mut("skills").and_then(|s| s.as_table_mut()) else {
        bail!(
            "skill \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        )
    };

    if !skills.contains_key(&args.name) {
        bail!(
            "skill \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        );
    }

    skills.remove(&args.name);

    if skills.is_empty() {
        doc.as_table_mut().remove("skills");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let label = format!("skill \"{}\"", args.name.cyan());
    if args.delete {
        try_delete_source(project_dir, source_path.as_deref(), &label, true)?;
    }

    print_removed(&label);

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(theta_cli::SyncArgs { force: true }, manifest_path)?;
    }
    Ok(())
}

fn rm_subagent(args: RmSubagentArgs, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;

    let project_dir = project_dir(manifest_path)?;

    // typed read to extract the ref path before mutation
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    let matched_sub = manifest
        .subagents
        .as_ref()
        .and_then(|subs| subs.iter().find(|s| s.name == args.name));
    let ref_path = matched_sub.and_then(|s| s.agent_ref.as_ref().map(|p| p.as_str().to_string()));
    let prompt_path = matched_sub.and_then(|s| s.prompt_path.clone());

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let Some(subagents) = doc.get("subagents").and_then(|s| s.as_array_of_tables()) else {
        bail!(
            "subagent \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        )
    };

    let mut found_idx = None;
    for (i, sub) in subagents.iter().enumerate() {
        if let Some(name) = sub.get("name").and_then(|n| n.as_str()) {
            if name == args.name {
                found_idx = Some(i);
                break;
            }
        }
    }

    let idx = found_idx.ok_or_else(|| {
        anyhow::anyhow!(
            "subagent \"{}\" is not registered in {}",
            args.name,
            manifest_path.display()
        )
    })?;

    let arr = doc["subagents"]
        .as_array_of_tables_mut()
        .context("[subagents] is not an array of tables")?;
    arr.remove(idx);

    if arr.is_empty() {
        doc.as_table_mut().remove("subagents");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let label = format!("subagent \"{}\"", args.name.cyan());
    if args.delete {
        try_delete_source(project_dir, ref_path.as_deref(), &label, false)?;
        try_delete_source(project_dir, prompt_path.as_deref(), &label, false)?;
    }

    print_removed(&label);

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(theta_cli::SyncArgs { force: true }, manifest_path)?;
    }
    Ok(())
}

fn rm_store(args: RmStoreArgs) -> Result<()> {
    let store = theta_store::StoreHandle::open()?;
    store.unregister(args.kind, &args.name)?;

    anstream::eprintln!(
        "{} {} '{}' from system store",
        "unregistered".red().bold(),
        args.kind,
        args.name.cyan(),
    );
    Ok(())
}
