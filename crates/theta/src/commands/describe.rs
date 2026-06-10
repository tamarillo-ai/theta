//! `theta describe` — read or set the agent description.

#![allow(clippy::print_stdout)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{DescribeArgs, OutputFormat};
use theta_manifest::{
    parse_manifest, read_document, read_manifest, set_value_strict, write_document,
};
use theta_schema::{Diagnostic, Validate};
use theta_static::is_placeholder_description;

use super::output::{present, present_error};

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
pub(crate) struct DescribeOutcome {
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
    let new_description = args.description.or(args.set);

    match new_description {
        None => read_description(manifest_path, args.rules, output_format),
        Some(desc) => write_description(manifest_path, &desc, output_format),
    }
}

fn read_description(
    manifest_path: &Path,
    show_rules: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let placeholder = is_placeholder_description(&manifest.agent.description);
    let description = if placeholder {
        None
    } else {
        Some(manifest.agent.description.clone())
    };

    let rules_out = if show_rules {
        manifest
            .instructions
            .as_ref()
            .and_then(|i| i.rules.as_ref())
            .filter(|r| !r.is_empty())
            .map(|r| {
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

    let outcome = DescribeOutcome {
        mode: DescribeMode::Read,
        description,
        rules: rules_out,
    };
    let manifest_path_owned: PathBuf = manifest_path.to_path_buf();
    let manifest_for_render = manifest;
    present(&["describe"], output_format, outcome, vec![], move |o| {
        if let Some(desc) = &o.description {
            println!("{desc}");
        } else {
            anstream::eprintln!(
                "{} no description set - edit {} or run `theta describe \"what your agent does\"`",
                "error".red().bold(),
                manifest_path_owned.display().cyan(),
            );
        }
        if show_rules {
            match manifest_for_render
                .instructions
                .as_ref()
                .and_then(|i| i.rules.as_ref())
                .filter(|r| !r.is_empty())
            {
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
    })
}

fn write_description(
    manifest_path: &Path,
    description: &str,
    output_format: OutputFormat,
) -> Result<()> {
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
        let outcome = DescribeOutcome {
            mode: DescribeMode::Write,
            description: Some(description.to_string()),
            rules: None,
        };
        let diags_for_render = desc_diags.clone();
        return present_error(
            &["describe"],
            output_format,
            outcome,
            desc_diags,
            move |_| {
                super::report_diagnostics(&diags_for_render);
            },
            anyhow!("description rejected - file not modified"),
        );
    }

    write_document(manifest_path, &doc)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;

    let outcome = DescribeOutcome {
        mode: DescribeMode::Write,
        description: Some(description.to_string()),
        rules: None,
    };
    let manifest_path_owned: PathBuf = manifest_path.to_path_buf();
    present(&["describe"], output_format, outcome, vec![], move |_| {
        anstream::eprintln!(
            "{} description in {}",
            "updated".green().bold(),
            manifest_path_owned.display().cyan(),
        );
    })
}
