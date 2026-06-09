//! `theta describe` — read or set the agent description.

#![allow(clippy::print_stdout)]

use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use theta_args::DescribeArgs;
use theta_manifest::{
    parse_manifest, read_document, read_manifest, set_value_strict, write_document,
};
use theta_schema::Validate;
use theta_static::is_placeholder_description;

pub(crate) fn execute(args: DescribeArgs, manifest_path: &Path) -> Result<()> {
    super::require_manifest(manifest_path)?;

    // resolve: positional arg takes priority, then --set, else read mode
    let new_description = args.description.or(args.set);

    match new_description {
        None => read_description(manifest_path, args.rules),
        Some(desc) => write_description(manifest_path, &desc),
    }
}

fn read_description(manifest_path: &Path, show_rules: bool) -> Result<()> {
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if is_placeholder_description(&manifest.agent.description) {
        anstream::eprintln!(
            "{} no description set - edit {} or run `theta describe \"what your agent does\"`",
            "error".red().bold(),
            manifest_path.display().cyan(),
        );
    } else {
        println!("{}", manifest.agent.description);
    }

    if show_rules {
        let rules = manifest
            .instructions
            .as_ref()
            .and_then(|i| i.rules.as_ref())
            .filter(|r| !r.is_empty());

        match rules {
            Some(rules) => {
                println!();
                println!("{}", "rules:".bold());
                for (name, rule) in rules {
                    println!("{}", super::list::format_rule_line(name, rule));
                }
            }
            None => {
                anstream::eprintln!("{} no rules registered", "info".blue().bold());
            }
        }
    }

    Ok(())
}

fn write_description(manifest_path: &Path, description: &str) -> Result<()> {
    let mut doc = read_document(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    set_value_strict(
        &mut doc,
        &["agent", "description"],
        toml_edit::Value::from(description),
    )
    .context("[agent] table not found - is this a valid theta.toml?")?;

    let manifest =
        parse_manifest(&doc.to_string()).with_context(|| "mutated document failed to parse")?;

    let mut diags = Vec::new();
    manifest.agent.validate(&mut diags);

    let desc_diags: Vec<_> = diags
        .into_iter()
        .filter(|d| d.path == "[agent].description")
        .collect();

    let (errors, _) = super::report_diagnostics(&desc_diags);
    if errors > 0 {
        anyhow::bail!("description rejected - file not modified");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    anstream::eprintln!(
        "{} description in {}",
        "updated".green().bold(),
        manifest_path.display().cyan(),
    );
    Ok(())
}
