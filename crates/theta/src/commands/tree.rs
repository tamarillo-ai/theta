//! `theta tree` — print dependency trees.
//!
//! Currently shows the subagent graph. Designed to be extensible for
//! skill dependencies when those exist.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{OutputFormat, TreeArgs};
use theta_manifest::read_manifest;
use theta_schema::{CommandOutput, Diagnostic};

use super::{project_dir, require_manifest};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct TreeNode {
    pub name: String,
    pub mode: Option<String>,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct TreeOutput {
    pub tree: TreeNode,
}

pub(crate) fn execute(
    _args: TreeArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    require_manifest(manifest_path)?;

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let project_dir = project_dir(manifest_path)?;

    let graph = theta_install::walk_subagent_graph(&manifest, project_dir);

    // build adjacency list: parent name --> children (name, mode)
    let mut edges: BTreeMap<String, Vec<(String, &str)>> = BTreeMap::new();
    for agent in &graph.agents {
        edges
            .entry(agent.declared_by.clone())
            .or_default()
            .push((agent.name.clone(), "ref"));
    }
    if let Some(ref subagents) = manifest.subagents {
        for sub in subagents {
            if sub.agent_ref.is_none() {
                edges
                    .entry(manifest.agent.name.clone())
                    .or_default()
                    .push((sub.name.clone(), "inline"));
            }
        }
    }

    if matches!(output_format, OutputFormat::Json) {
        let tree = build_tree(&manifest.agent.name, None, &edges);
        let diagnostics: Vec<Diagnostic> = graph
            .warnings
            .iter()
            .map(|w| Diagnostic::warn("[subagents]", w.message.clone()))
            .collect();
        let mut env = CommandOutput::ok(["tree"], TreeOutput { tree });
        env.diagnostics = diagnostics;
        env.print_json()?;
        return Ok(());
    }

    for w in &graph.warnings {
        anstream::eprintln!("{} {}", "warn".yellow().bold(), w.message);
    }

    anstream::eprintln!("{}", manifest.agent.name.bold());
    print_children(&manifest.agent.name, &edges, "");

    if edges.is_empty() {
        anstream::eprintln!("  {}", "(no subagents)".dimmed());
    }

    Ok(())
}

fn build_tree(
    name: &str,
    mode: Option<&str>,
    edges: &BTreeMap<String, Vec<(String, &str)>>,
) -> TreeNode {
    let children = edges
        .get(name)
        .map(|kids| {
            kids.iter()
                .map(|(child_name, child_mode)| build_tree(child_name, Some(child_mode), edges))
                .collect()
        })
        .unwrap_or_default();

    TreeNode {
        name: name.to_string(),
        mode: mode.map(ToString::to_string),
        children,
    }
}

fn print_children(parent: &str, edges: &BTreeMap<String, Vec<(String, &str)>>, prefix: &str) {
    let Some(children) = edges.get(parent) else {
        return;
    };
    for (i, (name, mode)) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let next_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
        anstream::eprintln!(
            "{prefix}{connector}{} {}",
            name.cyan(),
            format!("({mode})").dimmed()
        );
        print_children(name, edges, &next_prefix);
    }
}
