//! `theta rm` — remove rules, tools, skills, or subagents from the manifest.

use std::path::Path;

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use theta_args::{
    OutputFormat, RmCommand, RmNamespace, RmRuleArgs, RmSkillArgs, RmStoreArgs, RmSubagentArgs,
    RmSystemArgs, RmToolArgs,
};
use theta_manifest::{read_document, read_manifest, write_document};
use theta_schema::CommandOutput;
use theta_static::is_default_manifest;

use super::output::{EntityKind, MutationKind, MutationOutput};
use super::{project_dir, require_manifest};

pub(crate) fn dispatch(
    ns: RmNamespace,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    match ns.command {
        RmCommand::Rule(args) => rm_rule(args, output_format, manifest_path),
        RmCommand::System(args) => rm_system(args, output_format, manifest_path),
        RmCommand::Tool(args) => rm_tool(args, output_format, manifest_path),
        RmCommand::Skill(args) => rm_skill(args, output_format, manifest_path),
        RmCommand::Subagent(args) => rm_subagent(args, output_format, manifest_path),
        RmCommand::Store(args) => rm_store(args, output_format),
    }
}

fn emit_rm_envelope(
    verb_tail: &str,
    entity: EntityKind,
    name: Option<String>,
    files_deleted: Vec<PathBuf>,
) -> Result<()> {
    CommandOutput::ok(
        ["rm", verb_tail],
        MutationOutput {
            kind: MutationKind::Remove,
            entity,
            name,
            source: None,
            files_written: vec![],
            files_deleted,
        },
    )
    .print_json()?;
    Ok(())
}

fn print_removed(label: &str) {
    anstream::eprintln!("{} {}", "removed".red().bold(), label);
}

fn rm_rule(args: RmRuleArgs, output_format: OutputFormat, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

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

    let mut files_deleted = Vec::new();
    let label = format!("rule \"{}\"", args.name.cyan());
    if args.delete {
        if let Some(ref src) = source_path {
            let full = project_dir.join(src);
            if full.exists() {
                fs_err::remove_file(&full)
                    .with_context(|| format!("failed to delete {}", full.display()))?;
                files_deleted.push(full);
            }
        }
    }

    if json {
        emit_rm_envelope(
            "rule",
            EntityKind::Rule,
            Some(args.name.clone()),
            files_deleted,
        )?;
    } else {
        if args.delete {
            for p in &files_deleted {
                anstream::eprintln!(
                    "{} {} and deleted {}",
                    "removed".red().bold(),
                    label,
                    p.display().to_string().dimmed(),
                );
            }
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
    }

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(
            theta_args::SyncArgs { force: true },
            OutputFormat::Human,
            manifest_path,
        )?;
    }
    Ok(())
}

fn rm_system(args: RmSystemArgs, output_format: OutputFormat, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

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

    let mut files_deleted = Vec::new();
    let label = "system prompt";
    if args.delete {
        if let Some(ref src) = source_path {
            let full = project_dir.join(src);
            if full.exists() {
                fs_err::remove_file(&full)
                    .with_context(|| format!("failed to delete {}", full.display()))?;
                files_deleted.push(full);
            }
        }
    }

    if json {
        emit_rm_envelope("system", EntityKind::System, None, files_deleted)?;
    } else {
        if args.delete {
            for p in &files_deleted {
                anstream::eprintln!(
                    "{} {} and deleted {}",
                    "removed".red().bold(),
                    label,
                    p.display().to_string().dimmed(),
                );
            }
        }
        print_removed(label);
    }

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(
            theta_args::SyncArgs { force: true },
            OutputFormat::Human,
            manifest_path,
        )?;
    }
    Ok(())
}

fn rm_tool(args: RmToolArgs, output_format: OutputFormat, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

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

    if json {
        emit_rm_envelope("tool", EntityKind::Tool, Some(args.name.clone()), vec![])?;
    } else {
        print_removed(&format!("tool \"{}\"", args.name.cyan()));
    }
    Ok(())
}

fn rm_skill(args: RmSkillArgs, output_format: OutputFormat, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

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

    let mut files_deleted = Vec::new();
    let label = format!("skill \"{}\"", args.name.cyan());
    if args.delete {
        if let Some(ref src) = source_path {
            let full = project_dir.join(src);
            if full.exists() {
                fs_err::remove_dir_all(&full)
                    .with_context(|| format!("failed to delete {}", full.display()))?;
                files_deleted.push(full);
            }
        }
    }

    if json {
        emit_rm_envelope(
            "skill",
            EntityKind::Skill,
            Some(args.name.clone()),
            files_deleted,
        )?;
    } else {
        if args.delete {
            for p in &files_deleted {
                anstream::eprintln!(
                    "{} {} and deleted {}",
                    "removed".red().bold(),
                    label,
                    p.display().to_string().dimmed(),
                );
            }
        }
        print_removed(&label);
    }

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(
            theta_args::SyncArgs { force: true },
            OutputFormat::Human,
            manifest_path,
        )?;
    }
    Ok(())
}

fn rm_subagent(
    args: RmSubagentArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

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

    let mut files_deleted = Vec::new();
    let label = format!("subagent \"{}\"", args.name.cyan());
    if args.delete {
        for maybe in [ref_path.as_deref(), prompt_path.as_deref()]
            .into_iter()
            .flatten()
        {
            let full = project_dir.join(maybe);
            if full.exists() {
                fs_err::remove_file(&full)
                    .with_context(|| format!("failed to delete {}", full.display()))?;
                files_deleted.push(full);
            }
        }
    }

    if json {
        emit_rm_envelope(
            "subagent",
            EntityKind::Subagent,
            Some(args.name.clone()),
            files_deleted,
        )?;
    } else {
        if args.delete {
            for p in &files_deleted {
                anstream::eprintln!(
                    "{} {} and deleted {}",
                    "removed".red().bold(),
                    label,
                    p.display().to_string().dimmed(),
                );
            }
        }
        print_removed(&label);
    }

    if !args.no_sync && is_default_manifest(manifest_path) {
        crate::commands::sync::execute(
            theta_args::SyncArgs { force: true },
            OutputFormat::Human,
            manifest_path,
        )?;
    }
    Ok(())
}

fn rm_store(args: RmStoreArgs, output_format: OutputFormat) -> Result<()> {
    let json = matches!(output_format, OutputFormat::Json);
    let store = theta_store::StoreHandle::open()?;
    store.unregister(args.kind, &args.name)?;

    if json {
        let entity = match args.kind {
            theta_static::StoreResourceKind::Agent => EntityKind::Agent,
            theta_static::StoreResourceKind::Skill => EntityKind::Skill,
            theta_static::StoreResourceKind::Rule => EntityKind::Rule,
            _ => EntityKind::Agent,
        };
        CommandOutput::ok(
            ["rm", "store"],
            MutationOutput {
                kind: MutationKind::Unregister,
                entity,
                name: Some(args.name.clone()),
                source: None,
                files_written: vec![],
                files_deleted: vec![],
            },
        )
        .print_json()?;
    } else {
        anstream::eprintln!(
            "{} {} '{}' from system store",
            "unregistered".red().bold(),
            args.kind,
            args.name.cyan(),
        );
    }
    Ok(())
}
