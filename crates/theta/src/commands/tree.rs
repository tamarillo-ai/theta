//! `theta tree` — print dependency trees.
//!
//! Currently shows the subagent graph. Designed to be extensible for
//! skill dependencies when those exist.

#![allow(clippy::print_stdout)]

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use theta_cli::{OutputFormat, TreeArgs};
use theta_manifest::read_manifest;

use super::{project_dir, require_manifest};

pub(crate) fn execute(args: TreeArgs, manifest_path: &Path) -> Result<()> {
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

    if matches!(args.output_format, OutputFormat::Json) {
        let tree = build_json_tree(&manifest.agent.name, &edges);
        let warnings: Vec<&str> = graph.warnings.iter().map(|w| w.message.as_str()).collect();
        let output = serde_json::json!({
            "tree": tree,
            "warnings": warnings,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
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

fn build_json_tree(name: &str, edges: &BTreeMap<String, Vec<(String, &str)>>) -> serde_json::Value {
    let children: Vec<serde_json::Value> = edges
        .get(name)
        .map(|kids| {
            kids.iter()
                .map(|(child_name, mode)| {
                    let mut node = build_json_tree(child_name, edges);
                    node["mode"] = serde_json::Value::String(mode.to_string());
                    node
                })
                .collect()
        })
        .unwrap_or_default();

    serde_json::json!({
        "name": name,
        "children": children,
    })
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
