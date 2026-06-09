//! `theta add subagent` — scaffold or register a subagent in the manifest.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use owo_colors::OwoColorize;
use theta_args::AddSubagentArgs;
use theta_manifest::{parse_manifest, read_document, write_document};
use theta_schema::Validate;

use crate::commands::{project_dir, report_diagnostics, require_manifest};

enum SubagentIntent {
    /// No source flags --> scaffold `<project>/subagents/<name>.md` + register.
    CreateAndRegister,
    /// `--prompt-path` --> validate the file, register it
    RegisterExisting { prompt_path: PathBuf },
    /// `--agent-ref` --> existing ref behavior
    RegisterRef { agent_ref: PathBuf },
    /// `--description-only` --> no prompt file, no ref
    DescriptionOnly,
}

fn resolve_intent(args: &AddSubagentArgs) -> SubagentIntent {
    if let Some(ref r) = args.agent_ref {
        SubagentIntent::RegisterRef {
            agent_ref: r.clone(),
        }
    } else if let Some(ref p) = args.prompt_path {
        SubagentIntent::RegisterExisting {
            prompt_path: p.clone(),
        }
    } else if args.description_only {
        SubagentIntent::DescriptionOnly
    } else {
        SubagentIntent::CreateAndRegister
    }
}

pub(super) fn execute(args: AddSubagentArgs, manifest_path: &Path) -> Result<()> {
    require_manifest(manifest_path)?;
    let project_dir = project_dir(manifest_path)?;
    let intent = resolve_intent(&args);

    // description is required for all non-ref modes
    if !matches!(intent, SubagentIntent::RegisterRef { .. }) && args.description.is_none() {
        bail!("--description is required for inline subagents (or use --agent-ref for ref mode)");
    }

    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    reject_duplicate(&doc, &args.name, manifest_path)?;

    let (entry, scaffolded) = match intent {
        SubagentIntent::RegisterRef { ref agent_ref } => {
            validate_ref_file(agent_ref, project_dir, manifest_path)?;
            (build_ref_entry(&args.name, agent_ref, project_dir), None)
        }
        SubagentIntent::RegisterExisting { ref prompt_path } => {
            validate_prompt_path(prompt_path, project_dir)?;
            let rel = theta_static::rel_string(prompt_path, project_dir);
            (build_inline_entry(&args, Some(&rel)), None)
        }
        SubagentIntent::CreateAndRegister => {
            let file_path =
                scaffold_subagent_file(&args.name, args.description.as_deref(), project_dir)?;
            let rel = theta_static::rel_string(&file_path, project_dir);
            (build_inline_entry(&args, Some(&rel)), Some(file_path))
        }
        SubagentIntent::DescriptionOnly => (build_inline_entry(&args, None), None),
    };

    append_and_validate(&mut doc, entry)?;

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let mode_label = match intent {
        SubagentIntent::RegisterRef { .. } => "ref",
        _ => "inline",
    };
    anstream::eprintln!(
        "{} subagent \"{}\" ({})",
        "registered".green().bold(),
        args.name.cyan(),
        mode_label,
    );
    if let Some(path) = scaffolded {
        anstream::eprintln!("  {} {}", "created".green().bold(), path.display(),);
    }

    Ok(())
}

fn reject_duplicate(doc: &toml_edit::DocumentMut, name: &str, manifest_path: &Path) -> Result<()> {
    if let Some(subagents) = doc.get("subagents").and_then(|s| s.as_array_of_tables()) {
        for sub in subagents {
            if sub.get("name").and_then(|n| n.as_str()) == Some(name) {
                bail!(
                    "subagent \"{}\" is already registered in {}",
                    name,
                    manifest_path.display()
                );
            }
        }
    }
    Ok(())
}

fn validate_ref_file(ref_path: &Path, project_dir: &Path, manifest_path: &Path) -> Result<()> {
    let resolved = if ref_path.is_relative() {
        project_dir.join(ref_path)
    } else {
        ref_path.to_path_buf()
    };
    if !resolved.exists() {
        bail!("referenced file does not exist: {}", resolved.display());
    }
    if let (Ok(ref_canon), Ok(manifest_canon)) =
        (resolved.canonicalize(), manifest_path.canonicalize())
    {
        if ref_canon == manifest_canon {
            bail!(
                "subagent ref points to the manifest itself - self-referential subagents are not allowed"
            );
        }
    }
    let content = fs_err::read_to_string(&resolved)
        .with_context(|| format!("failed to read {}", resolved.display()))?;
    theta_manifest::parse_manifest(&content)
        .with_context(|| format!("{} is not a valid theta.toml", resolved.display()))?;
    Ok(())
}

fn validate_prompt_path(prompt_path: &Path, project_dir: &Path) -> Result<()> {
    let resolved = if prompt_path.is_relative() {
        project_dir.join(prompt_path)
    } else {
        prompt_path.to_path_buf()
    };
    if !resolved.exists() {
        bail!("prompt file does not exist: {}", resolved.display());
    }
    let name = resolved
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 path: {}", resolved.display()))?;
    if !name.ends_with(".md") {
        bail!("prompt_path must end in .md - got: {name}");
    }
    Ok(())
}

fn scaffold_subagent_file(
    name: &str,
    description: Option<&str>,
    project_dir: &Path,
) -> Result<PathBuf> {
    let dir = project_dir.join(theta_static::SUBAGENTS_DIR_NAME);
    fs_err::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    let file_path = dir.join(format!("{name}.md"));
    if file_path.exists() {
        bail!(
            "{} already exists - use --prompt-path to register it instead",
            file_path.display()
        );
    }

    let desc = description.unwrap_or("TODO: describe this subagent");
    let content = format!("# {name}\n\n{desc}\n");
    fs_err::write(&file_path, &content)
        .with_context(|| format!("failed to write {}", file_path.display()))?;
    Ok(file_path)
}

fn append_and_validate(doc: &mut toml_edit::DocumentMut, entry: toml_edit::Table) -> Result<()> {
    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    doc["subagents"]
        .as_array_of_tables_mut()
        .context("[subagents] is not an array of tables")?
        .push(entry);

    let manifest =
        parse_manifest(&doc.to_string()).with_context(|| "mutated document failed to parse")?;
    let mut diags = Vec::new();
    manifest.validate(&mut diags);

    let sub_diags: Vec<_> = diags
        .into_iter()
        .filter(|d| d.path.contains("[subagents"))
        .collect();
    let (errors, _) = report_diagnostics(&sub_diags);
    if errors > 0 {
        bail!("subagent rejected - manifest not modified");
    }
    Ok(())
}

fn build_ref_entry(name: &str, ref_path: &Path, project_dir: &Path) -> toml_edit::Table {
    let ref_rel = theta_static::rel_string(ref_path, project_dir);

    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value(name);
    entry["description"] = toml_edit::value("");
    entry["ref"] = toml_edit::value(&ref_rel);
    entry
}

fn build_inline_entry(args: &AddSubagentArgs, prompt_path_rel: Option<&str>) -> toml_edit::Table {
    let mut entry = toml_edit::Table::new();
    entry["name"] = toml_edit::value(&args.name);
    entry["description"] = toml_edit::value(args.description.as_deref().unwrap_or(""));

    if let Some(ref model) = args.model {
        entry["model"] = toml_edit::value(model.as_str());
    }
    if let Some(rel) = prompt_path_rel {
        entry["prompt_path"] = toml_edit::value(rel);
    }
    if let Some(ref tools) = args.tools {
        let mut arr = toml_edit::Array::new();
        for t in tools {
            arr.push(t.as_str());
        }
        entry["tools"] = toml_edit::value(arr);
    }
    if let Some(ref skills) = args.skills {
        let mut arr = toml_edit::Array::new();
        for s in skills {
            arr.push(s.as_str());
        }
        entry["skills"] = toml_edit::value(arr);
    }
    entry
}
