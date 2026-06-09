//! `theta describe` — read or set the agent description.

#![allow(clippy::print_stdout)]

use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{DescribeArgs, OutputFormat};
use theta_manifest::{
    parse_manifest, read_document, read_manifest, set_value_strict, write_document,
};
use theta_schema::{CommandFailure, CommandOutput, Diagnostic, Validate};
use theta_static::is_placeholder_description;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DescribeMode {
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct DescribeRule {
    pub name: String,
    pub apply: String,
    pub source: String,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct DescribeOutput {
    pub mode: DescribeMode,
    pub description: Option<String>,
    pub rules: Option<Vec<DescribeRule>>,
}

pub(crate) fn execute(
    args: DescribeArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    super::require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

    // resolve: positional arg takes priority, then --set, else read mode
    let new_description = args.description.or(args.set);

    match new_description {
        None => read_description(manifest_path, args.rules, json),
        Some(desc) => write_description(manifest_path, &desc, json),
    }
}

fn read_description(manifest_path: &Path, show_rules: bool, json: bool) -> Result<()> {
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let placeholder = is_placeholder_description(&manifest.agent.description);
    let description = if placeholder {
        None
    } else {
        Some(manifest.agent.description.clone())
    };

    let rules_out = if show_rules {
        let rules = manifest
            .instructions
            .as_ref()
            .and_then(|i| i.rules.as_ref())
            .filter(|r| !r.is_empty());
        rules.map(|r| {
            r.iter()
                .map(|(name, rule)| DescribeRule {
                    name: name.clone(),
                    apply: rule.apply.as_str().to_string(),
                    source: rule.src.display_compact(),
                    summary: rule.summary.clone(),
                })
                .collect::<Vec<_>>()
        })
    } else {
        None
    };

    if json {
        CommandOutput::ok(
            ["describe"],
            DescribeOutput {
                mode: DescribeMode::Read,
                description,
                rules: rules_out,
            },
        )
        .print_json()?;
        return Ok(());
    }

    if placeholder {
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

fn write_description(manifest_path: &Path, description: &str, json: bool) -> Result<()> {
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

    let desc_diags: Vec<Diagnostic> = diags
        .into_iter()
        .filter(|d| d.path == "[agent].description")
        .collect();

    let errors = desc_diags
        .iter()
        .filter(|d| matches!(d.level, theta_schema::DiagLevel::Error))
        .count();

    if errors > 0 {
        if json {
            CommandOutput::error(
                ["describe"],
                DescribeOutput {
                    mode: DescribeMode::Write,
                    description: Some(description.to_string()),
                    rules: None,
                },
                desc_diags,
            )
            .print_json()?;
            return Err(CommandFailure.into());
        }
        let (_errors, _warnings) = super::report_diagnostics(&desc_diags);
        anyhow::bail!("description rejected - file not modified");
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    if json {
        CommandOutput::ok(
            ["describe"],
            DescribeOutput {
                mode: DescribeMode::Write,
                description: Some(description.to_string()),
                rules: None,
            },
        )
        .print_json()?;
    } else {
        anstream::eprintln!(
            "{} description in {}",
            "updated".green().bold(),
            manifest_path.display().cyan(),
        );
    }
    Ok(())
}
