//! Agent dependency graph walker.
//!
//! TODO: this module is preserved for the future of a peer-like feature, which will
//! model bidirectional agent collaboration.
//!
//! Currently unused by lock or materialize (depth-1 subagents are handled
//! directly from the lock). Used by `theta tree` for display.
//!
//! Walks the ref subagent graph from a root manifest breadth-first and
//! returns a flat list of all unique agents discovered.
//! Handles cycles by tracking visited canonical paths (silent skip).
//! Handles name collisions with first-wins + warning.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ResolvedSubagent {
    pub name: String,
    pub manifest_path: PathBuf,
    pub project_dir: PathBuf,
    pub manifest: theta_schema::ThetaManifest,
    pub declared_by: String,
}

#[derive(Debug, Clone)]
pub struct GraphWarning {
    pub message: String,
}

pub struct SubagentGraph {
    pub agents: Vec<ResolvedSubagent>,
    pub warnings: Vec<GraphWarning>,
}

struct QueueItem {
    parent_name: String,
    manifest: theta_schema::ThetaManifest,
    project_dir: PathBuf,
}

pub fn walk_subagent_graph(
    root_manifest: &theta_schema::ThetaManifest,
    root_project_dir: &Path,
) -> SubagentGraph {
    let mut agents: Vec<ResolvedSubagent> = Vec::new();
    let mut warnings: Vec<GraphWarning> = Vec::new();

    let mut visited_paths: HashSet<PathBuf> = HashSet::new();
    let mut claimed_names: BTreeMap<String, String> = BTreeMap::new();

    let mut queue: VecDeque<QueueItem> = VecDeque::new();
    queue.push_back(QueueItem {
        parent_name: root_manifest.agent.name.clone(),
        manifest: root_manifest.clone(),
        project_dir: root_project_dir.to_path_buf(),
    });

    while let Some(item) = queue.pop_front() {
        let Some(ref subagents) = item.manifest.subagents else {
            continue;
        };

        for sub in subagents {
            let Some(ref agent_ref) = sub.agent_ref else {
                continue;
            };

            let child_manifest_path = item.project_dir.join(agent_ref.as_str());

            let canonical = match child_manifest_path.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    warnings.push(GraphWarning {
                        message: format!(
                            "subagent '{}': cannot resolve {}: {}",
                            sub.name,
                            child_manifest_path.display(),
                            e,
                        ),
                    });
                    continue;
                }
            };
            if visited_paths.contains(&canonical) {
                continue;
            }
            visited_paths.insert(canonical.clone());

            if let Some(existing_parent) = claimed_names.get(&sub.name) {
                warnings.push(GraphWarning {
                    message: format!(
                        "subagent '{}' declared by both '{}' and '{}' -- using {}'s version",
                        sub.name, existing_parent, item.parent_name, existing_parent,
                    ),
                });
                continue;
            }
            claimed_names.insert(sub.name.clone(), item.parent_name.clone());

            let child_content = match fs_err::read_to_string(&canonical) {
                Ok(c) => c,
                Err(e) => {
                    warnings.push(GraphWarning {
                        message: format!(
                            "subagent '{}': failed to read {}: {}",
                            sub.name,
                            canonical.display(),
                            e,
                        ),
                    });
                    continue;
                }
            };
            let child_manifest: theta_schema::ThetaManifest = match toml::from_str(&child_content) {
                Ok(m) => m,
                Err(e) => {
                    warnings.push(GraphWarning {
                        message: format!(
                            "subagent '{}': failed to parse {}: {}",
                            sub.name,
                            canonical.display(),
                            e,
                        ),
                    });
                    continue;
                }
            };

            let child_project_dir = canonical.parent().unwrap_or(Path::new(".")).to_path_buf();

            queue.push_back(QueueItem {
                parent_name: sub.name.clone(),
                manifest: child_manifest.clone(),
                project_dir: child_project_dir.clone(),
            });

            agents.push(ResolvedSubagent {
                name: sub.name.clone(),
                manifest_path: child_manifest_path,
                project_dir: child_project_dir,
                manifest: child_manifest,
                declared_by: item.parent_name.clone(),
            });
        }
    }

    SubagentGraph { agents, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theta_schema::minimal_manifest;

    #[test]
    fn empty_manifest_produces_empty_graph() {
        let manifest = minimal_manifest("root");
        let dir = tempfile::tempdir().unwrap();
        let graph = walk_subagent_graph(&manifest, dir.path());
        assert!(graph.agents.is_empty());
        assert!(graph.warnings.is_empty());
    }

    #[test]
    fn inline_subagents_are_skipped() {
        let mut manifest = minimal_manifest("root");
        manifest.subagents = Some(vec![theta_schema::Subagent {
            name: "helper".into(),
            description: Some("inline helper".into()),
            agent_ref: None,
            prompt_path: None,
            model: None,
            tools: None,
            skills: None,
        }]);
        let dir = tempfile::tempdir().unwrap();
        let graph = walk_subagent_graph(&manifest, dir.path());
        assert!(graph.agents.is_empty());
    }

    #[test]
    fn ref_subagent_is_resolved() {
        let dir = tempfile::tempdir().unwrap();
        let child_dir = dir.path().join("agents/scout");
        fs_err::create_dir_all(&child_dir).unwrap();
        fs_err::write(
            child_dir.join("theta.toml"),
            r#"
[theta]
schema = "2026-04"
[agent]
name = "scout"
description = "a scout"
"#,
        )
        .unwrap();

        let mut manifest = minimal_manifest("root");
        manifest.subagents = Some(vec![theta_schema::Subagent {
            name: "scout".into(),
            description: Some(String::new()),
            agent_ref: Some(theta_schema::LocalPathRef::from("agents/scout/theta.toml")),
            prompt_path: None,
            model: None,
            tools: None,
            skills: None,
        }]);

        let graph = walk_subagent_graph(&manifest, dir.path());
        assert_eq!(graph.agents.len(), 1);
        assert_eq!(graph.agents[0].name, "scout");
        assert_eq!(graph.agents[0].manifest.agent.description, "a scout");
        assert_eq!(graph.agents[0].declared_by, "root");
    }

    #[test]
    fn cycle_is_silently_skipped() {
        let dir = tempfile::tempdir().unwrap();

        // agent A refs agent B, agent B refs agent A
        let a_dir = dir.path().join("agents/a");
        let b_dir = dir.path().join("agents/b");
        fs_err::create_dir_all(&a_dir).unwrap();
        fs_err::create_dir_all(&b_dir).unwrap();

        // B's theta.toml refs A
        fs_err::write(
            b_dir.join("theta.toml"),
            format!(
                r#"
[theta]
schema = "2026-04"
[agent]
name = "b"
description = "agent b"
[[subagents]]
name = "a"
description = ""
ref = "{}"
"#,
                a_dir.join("theta.toml").display()
            ),
        )
        .unwrap();

        // A's theta.toml refs B
        fs_err::write(
            a_dir.join("theta.toml"),
            format!(
                r#"
[theta]
schema = "2026-04"
[agent]
name = "a"
description = "agent a"
[[subagents]]
name = "b"
description = ""
ref = "{}"
"#,
                b_dir.join("theta.toml").display()
            ),
        )
        .unwrap();

        // root refs A
        let mut manifest = minimal_manifest("root");
        manifest.subagents = Some(vec![theta_schema::Subagent {
            name: "a".into(),
            description: Some(String::new()),
            agent_ref: Some(theta_schema::LocalPathRef::from("agents/a/theta.toml")),
            prompt_path: None,
            model: None,
            tools: None,
            skills: None,
        }]);

        let graph = walk_subagent_graph(&manifest, dir.path());
        // should see A and B, each once. no infinite loop
        assert_eq!(graph.agents.len(), 2);
        let names: Vec<&str> = graph.agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
        assert!(graph.warnings.is_empty());
    }

    #[test]
    fn name_collision_warns_and_keeps_first() {
        let dir = tempfile::tempdir().unwrap();

        // two parents each declare a subagent named "helper"
        let helper1_dir = dir.path().join("agents/group1/helper");
        let helper2_dir = dir.path().join("agents/group2/helper");
        let parent1_dir = dir.path().join("agents/parent1");
        let parent2_dir = dir.path().join("agents/parent2");
        fs_err::create_dir_all(&helper1_dir).unwrap();
        fs_err::create_dir_all(&helper2_dir).unwrap();
        fs_err::create_dir_all(&parent1_dir).unwrap();
        fs_err::create_dir_all(&parent2_dir).unwrap();

        let helper_toml = r#"
[theta]
schema = "2026-04"
[agent]
name = "helper"
description = "a helper"
"#;
        fs_err::write(helper1_dir.join("theta.toml"), helper_toml).unwrap();
        fs_err::write(helper2_dir.join("theta.toml"), helper_toml).unwrap();

        // parent1 refs helper1
        fs_err::write(
            parent1_dir.join("theta.toml"),
            format!(
                r#"
[theta]
schema = "2026-04"
[agent]
name = "parent1"
description = "p1"
[[subagents]]
name = "helper"
description = ""
ref = "{}"
"#,
                helper1_dir.join("theta.toml").display()
            ),
        )
        .unwrap();

        // parent2 refs helper2
        fs_err::write(
            parent2_dir.join("theta.toml"),
            format!(
                r#"
[theta]
schema = "2026-04"
[agent]
name = "parent2"
description = "p2"
[[subagents]]
name = "helper"
description = ""
ref = "{}"
"#,
                helper2_dir.join("theta.toml").display()
            ),
        )
        .unwrap();

        // root refs parent1 and parent2
        let mut manifest = minimal_manifest("root");
        manifest.subagents = Some(vec![
            theta_schema::Subagent {
                name: "parent1".into(),
                description: Some(String::new()),
                agent_ref: Some(theta_schema::LocalPathRef::from(
                    "agents/parent1/theta.toml",
                )),
                prompt_path: None,
                model: None,
                tools: None,
                skills: None,
            },
            theta_schema::Subagent {
                name: "parent2".into(),
                description: Some(String::new()),
                agent_ref: Some(theta_schema::LocalPathRef::from(
                    "agents/parent2/theta.toml",
                )),
                prompt_path: None,
                model: None,
                tools: None,
                skills: None,
            },
        ]);

        let graph = walk_subagent_graph(&manifest, dir.path());
        // parent1, parent2, helper (from parent1). helper from parent2 is a collision
        let names: Vec<&str> = graph.agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"parent1"));
        assert!(names.contains(&"parent2"));
        assert!(names.contains(&"helper"));
        // exactly one warning about the collision
        assert_eq!(graph.warnings.len(), 1);
        assert!(graph.warnings[0].message.contains("helper"));
        assert!(graph.warnings[0].message.contains("parent1"));
        assert!(graph.warnings[0].message.contains("parent2"));
    }
}
