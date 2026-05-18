//! Ref resolution — validates that manifest-declared resources are reachable.
//!
//! Local and system-store refs are checked on disk. Git refs produce
//! "run `theta sync`" hints.

use std::path::Path;

use theta_dirs::data_dir;
use theta_schema::{
    Diagnostic, LocalOrGitRef, ResolvedRefKey, ResolvedRefs, SourceRef, ThetaManifest,
    ValidateContent,
};
use theta_static::{GIT_UNRESOLVED_CODE, SKILL_FILE_NAME, SystemStoreLayout};

#[derive(Debug, Clone)]
struct ManifestRef {
    context: &'static str,
    label: String,
    resolved_key: ResolvedRefKey,
    resource: ResourceRef,
}

#[derive(Debug, Clone)]
enum ResourceRef {
    Local {
        path: String,
        is_dir: bool,
    },
    Git {
        url: String,
        git_ref: Option<String>,
        file: Option<String>,
    },
    Store {
        name: String,
        is_dir: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum RefStatus {
    Reachable,
    NotFound(String),
    Unchecked(String),
}

fn collect_manifest_refs(manifest: &ThetaManifest) -> Vec<ManifestRef> {
    let mut refs = Vec::new();

    if let Some(ref instructions) = manifest.instructions {
        if let Some(ref system) = instructions.system {
            refs.push(ManifestRef {
                context: "[instructions].system",
                label: "system prompt".to_string(),
                resolved_key: ResolvedRefKey::instructions_system(),
                resource: ResourceRef::Local {
                    path: system.as_str().to_string(),
                    is_dir: false,
                },
            });
        }

        if let Some(ref rules) = instructions.rules {
            for (name, rule) in rules {
                let resource = match &rule.src {
                    LocalOrGitRef::Local(path) => ResourceRef::Local {
                        path: path.as_str().to_string(),
                        is_dir: false,
                    },
                    LocalOrGitRef::Git {
                        git,
                        branch,
                        tag,
                        rev,
                        file,
                        ..
                    } => ResourceRef::Git {
                        url: git.clone(),
                        git_ref: branch
                            .clone()
                            .or_else(|| tag.clone())
                            .or_else(|| rev.clone()),
                        file: Some(file.clone()),
                    },
                    LocalOrGitRef::System { system } => ResourceRef::Store {
                        name: system.clone(),
                        is_dir: false,
                    },
                    _ => continue,
                };

                refs.push(ManifestRef {
                    context: "[instructions.rules]",
                    label: format!("[instructions.rules.{name}]"),
                    resolved_key: ResolvedRefKey::instructions_rule(name.clone()),
                    resource,
                });
            }
        }
    }

    if let Some(ref skills) = manifest.skills {
        for (name, skill) in skills {
            let resource = match &skill.source {
                SourceRef::Path { path } => ResourceRef::Local {
                    path: path.clone(),
                    is_dir: true,
                },
                SourceRef::Git {
                    git,
                    branch,
                    tag,
                    rev,
                    subdirectory,
                    ..
                } => ResourceRef::Git {
                    url: git.clone(),
                    git_ref: branch
                        .clone()
                        .or_else(|| tag.clone())
                        .or_else(|| rev.clone()),
                    file: subdirectory.clone(),
                },
                SourceRef::System { system } => ResourceRef::Store {
                    name: system.clone(),
                    is_dir: true,
                },
                _ => continue,
            };

            refs.push(ManifestRef {
                context: "[skills]",
                label: format!("[skills.{name}]"),
                resolved_key: ResolvedRefKey::skill(name.clone()),
                resource,
            });
        }
    }

    if let Some(ref subagents) = manifest.subagents {
        for sub in subagents {
            if let Some(ref agent_ref) = sub.agent_ref {
                refs.push(ManifestRef {
                    context: "[[subagents]]",
                    label: format!("subagents.{} ref", sub.name),
                    resolved_key: ResolvedRefKey::subagent_ref(sub.name.clone()),
                    resource: ResourceRef::Local {
                        path: agent_ref.as_str().to_string(),
                        is_dir: false,
                    },
                });
            }
            if let Some(ref prompt_path) = sub.prompt_path {
                refs.push(ManifestRef {
                    context: "[[subagents]]",
                    label: format!("subagents.{} prompt_path", sub.name),
                    resolved_key: ResolvedRefKey::subagent_prompt(sub.name.clone()),
                    resource: ResourceRef::Local {
                        path: prompt_path.clone(),
                        is_dir: false,
                    },
                });
            }
        }
    }

    refs
}

pub(crate) fn check_refs(
    manifest: &ThetaManifest,
    project_dir: &Path,
    strict_materialization: bool,
    diags: &mut Vec<Diagnostic>,
) {
    let refs = collect_manifest_refs(manifest);
    for r in refs {
        match check_resource(&r.resource, project_dir) {
            RefStatus::Reachable => {}
            RefStatus::NotFound(msg) => {
                diags.push(Diagnostic::error(r.context, format!("{} {}", r.label, msg)));
            }
            RefStatus::Unchecked(msg) => {
                let is_instruction_ref = r.context.starts_with("[instructions");
                if strict_materialization && is_instruction_ref {
                    diags.push(Diagnostic::error(r.context, format!("{} {}", r.label, msg)));
                } else {
                    diags.push(Diagnostic::warn(r.context, format!("{} {}", r.label, msg)));
                }
            }
        }
    }
}

pub(crate) fn resolve_content(manifest: &ThetaManifest, project_dir: &Path) -> ResolvedRefs {
    let refs = collect_manifest_refs(manifest);
    let mut resolved = ResolvedRefs::new();
    for r in refs {
        let key = r.resolved_key;
        match r.resource {
            ResourceRef::Local { path, is_dir } => {
                let full = project_dir.join(&path);
                if !full.exists() {
                    resolved.insert_missing(key);
                    continue;
                }
                if is_dir {
                    if !full.is_dir() {
                        resolved.insert_error(
                            key,
                            format!("path exists but is not a directory: {path}"),
                        );
                        continue;
                    }
                    // for directory resources (skills), read the SKILL.md inside
                    let skill_md = full.join(SKILL_FILE_NAME);
                    match fs_err::read_to_string(&skill_md) {
                        Ok(content) => resolved.insert_resolved(key, content),
                        Err(err) => {
                            resolved.insert_error(
                                key,
                                format!("failed to read {path}/{SKILL_FILE_NAME}: {err}"),
                            );
                        }
                    }
                } else {
                    if !full.is_file() {
                        resolved
                            .insert_error(key, format!("path exists but is not a file: {path}"));
                        continue;
                    }
                    match fs_err::read_to_string(&full) {
                        Ok(content) => resolved.insert_resolved(key, content),
                        Err(err) => {
                            resolved.insert_error(key, format!("failed to read {path}: {err}"));
                        }
                    }
                }
            }
            ResourceRef::Git { .. } => {
                resolved.insert_deferred(key);
            }
            ResourceRef::Store { name, is_dir } => {
                let Some(data_dir) = data_dir() else {
                    resolved.insert_error(
                        key,
                        "could not determine system store directory".to_string(),
                    );
                    continue;
                };
                let store = SystemStoreLayout::new(&data_dir);
                if is_dir {
                    let skill_dir = store.skill(&name);
                    if !skill_dir.exists() {
                        resolved.insert_missing(key);
                        continue;
                    }
                    let skill_md = skill_dir.join(SKILL_FILE_NAME);
                    match fs_err::read_to_string(&skill_md) {
                        Ok(content) => resolved.insert_resolved(key, content),
                        Err(err) => resolved.insert_error(
                            key,
                            format!("failed to read store skill '{name}': {err}"),
                        ),
                    }
                } else {
                    let rule_path = store.rule(&name);
                    match fs_err::read_to_string(&rule_path) {
                        Ok(content) => resolved.insert_resolved(key, content),
                        Err(err) => resolved.insert_error(
                            key,
                            format!("failed to read store rule '{name}': {err}"),
                        ),
                    }
                }
            }
        }
    }
    resolved
}

fn check_resource(resource: &ResourceRef, project_dir: &Path) -> RefStatus {
    match resource {
        ResourceRef::Local { path, is_dir } => {
            let full = project_dir.join(path);
            if !full.exists() {
                RefStatus::NotFound(format!("not found: {path}"))
            } else if *is_dir {
                if !full.is_dir() {
                    RefStatus::NotFound(format!("path exists but is not a directory: {path}"))
                } else if !full.join(SKILL_FILE_NAME).exists() {
                    RefStatus::NotFound(format!("{SKILL_FILE_NAME} not found in {path}"))
                } else {
                    RefStatus::Reachable
                }
            } else if !full.is_file() {
                RefStatus::NotFound(format!("path exists but is not a file: {path}"))
            } else {
                RefStatus::Reachable
            }
        }
        ResourceRef::Git { url, git_ref, file } => {
            if !url.starts_with("https://")
                && !url.starts_with("git://")
                && !url.starts_with("ssh://")
                && !url.starts_with("http://")
            {
                return RefStatus::NotFound(format!("invalid git URL: {url}"));
            }
            let target = match (git_ref, file) {
                (Some(r), Some(f)) => format!("{url}#{r}:{f}"),
                (Some(r), None) => format!("{url}#{r}"),
                (None, Some(f)) => format!("{url}:{f}"),
                (None, None) => url.clone(),
            };
            RefStatus::Unchecked(format!(
                "{GIT_UNRESOLVED_CODE}: {target} -- run `theta sync` to fetch"
            ))
        }
        ResourceRef::Store { name, is_dir } => {
            let Some(data_dir) = data_dir() else {
                return RefStatus::NotFound(format!("system store unavailable for '{name}'"));
            };
            let store = SystemStoreLayout::new(&data_dir);
            if *is_dir {
                let skill_dir = store.skill(name);
                if !skill_dir.exists() {
                    RefStatus::NotFound(format!(
                        "store skill '{name}' not found - run `theta register skill {name}`"
                    ))
                } else if !skill_dir.join(SKILL_FILE_NAME).exists() {
                    RefStatus::NotFound(format!("store skill '{name}' is missing SKILL.md"))
                } else {
                    RefStatus::Reachable
                }
            } else {
                let rule_path = store.rule(name);
                if rule_path.exists() {
                    RefStatus::Reachable
                } else {
                    RefStatus::NotFound(format!(
                        "store rule '{name}' not found - run `theta register rule {name}`"
                    ))
                }
            }
        }
    }
}

/// Validate manifest-declared resources after materialization.
///
/// Bundles `check_refs` (reachability) + `resolve_content` (load) +
/// `manifest.validate_content` (semantic) into a single call. Runs in
/// `strict_materialization = true` mode.
pub(crate) fn validate_materialized(
    manifest: &ThetaManifest,
    project_dir: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    check_refs(manifest, project_dir, true, diags);
    let resolved = resolve_content(manifest, project_dir);
    manifest.validate_content("", &resolved, diags);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use theta_schema::{
        Agent, ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, Skill, Theta,
        ThetaManifest,
    };

    fn manifest_with_all_current_ref_sites() -> ThetaManifest {
        let mut rules = BTreeMap::new();
        rules.insert(
            "local-rule".to_string(),
            Rule {
                src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/local.md")),
                summary: None,
                description: None,
                apply: ApplyMode::Always,
                apply_to: None,
            },
        );
        rules.insert(
            "git-rule".to_string(),
            Rule {
                src: LocalOrGitRef::Git {
                    git: "https://example.com/rules.git".to_string(),
                    branch: Some("main".to_string()),
                    tag: None,
                    rev: None,
                    file: "rules/remote.md".to_string(),
                },
                summary: None,
                description: None,
                apply: ApplyMode::Always,
                apply_to: None,
            },
        );

        let mut skills = BTreeMap::new();
        skills.insert(
            "local-skill".to_string(),
            Skill {
                source: SourceRef::Path {
                    path: "skills/local-skill".to_string(),
                },
                tags: None,
                goal: None,
            },
        );
        skills.insert(
            "git-skill".to_string(),
            Skill {
                source: SourceRef::Git {
                    git: "https://example.com/skills.git".to_string(),
                    branch: None,
                    tag: Some("v1".to_string()),
                    rev: None,
                    subdirectory: Some("skill".to_string()),
                },
                tags: None,
                goal: None,
            },
        );
        ThetaManifest {
            theta: Theta {
                schema: theta_static::SCHEMA_VERSION.to_string(),
            },
            agent: Agent {
                name: "test-agent".to_string(),
                description: "Test agent".to_string(),
                version: None,
                authors: None,
                model: Some("claude-sonnet-4-20250514".to_string()),
                tags: None,
            },
            instructions: Some(Instructions {
                system: Some(LocalPathRef::from("instructions/system.md")),
                rules: Some(rules),
            }),
            tools: None,
            skills: Some(skills),
            subagents: None,
            harness: None,
            extras: None,
        }
    }

    #[test]
    fn collect_manifest_refs_covers_all_current_ref_sites() {
        let manifest = manifest_with_all_current_ref_sites();
        let refs = collect_manifest_refs(&manifest);
        let labels: Vec<_> = refs.iter().map(|r| r.label.as_str()).collect();

        assert_eq!(refs.len(), 5);
        assert_eq!(
            labels,
            vec![
                "system prompt",
                "[instructions.rules.git-rule]",
                "[instructions.rules.local-rule]",
                "[skills.git-skill]",
                "[skills.local-skill]",
            ]
        );
    }

    #[test]
    fn resolve_content_only_loads_local_refs() {
        let dir = tempfile::tempdir().unwrap();
        fs_err::create_dir_all(dir.path().join("instructions/rules")).unwrap();
        fs_err::create_dir_all(dir.path().join("skills/local-skill")).unwrap();
        fs_err::write(dir.path().join("instructions/system.md"), "system").unwrap();
        fs_err::write(dir.path().join("instructions/rules/local.md"), "local rule").unwrap();
        fs_err::write(dir.path().join("skills/local-skill/README.md"), "ignored").unwrap();

        let manifest = manifest_with_all_current_ref_sites();
        let resolved = resolve_content(&manifest, dir.path());

        assert_eq!(resolved.get_instructions_system(), Some("system"));
        assert_eq!(
            resolved.get_instructions_rule("local-rule"),
            Some("local rule")
        );
        assert_eq!(resolved.get_instructions_rule("git-rule"), None);
    }

    #[test]
    fn check_refs_warns_for_remote_refs() {
        let dir = tempfile::tempdir().unwrap();
        fs_err::create_dir_all(dir.path().join("instructions/rules")).unwrap();
        fs_err::write(dir.path().join("instructions/system.md"), "system").unwrap();
        fs_err::write(dir.path().join("instructions/rules/local.md"), "local rule").unwrap();

        let manifest = manifest_with_all_current_ref_sites();
        let mut diags = Vec::new();
        check_refs(&manifest, dir.path(), false, &mut diags);

        assert!(diags.iter().any(|d| {
            d.level == theta_schema::DiagLevel::Warn
                && d.path == "[instructions.rules]"
                && d.message.contains(GIT_UNRESOLVED_CODE)
                && d.message.contains("[instructions.rules.git-rule]")
        }));
        assert!(diags.iter().any(|d| {
            d.level == theta_schema::DiagLevel::Warn
                && d.path == "[skills]"
                && d.message.contains(GIT_UNRESOLVED_CODE)
                && d.message.contains("[skills.git-skill]")
        }));
    }

    #[test]
    fn check_refs_strict_materialization_errors_on_instructions_only() {
        let dir = tempfile::tempdir().unwrap();
        fs_err::create_dir_all(dir.path().join("instructions/rules")).unwrap();
        fs_err::write(dir.path().join("instructions/system.md"), "system").unwrap();
        fs_err::write(dir.path().join("instructions/rules/local.md"), "local rule").unwrap();

        let manifest = manifest_with_all_current_ref_sites();
        let mut diags = Vec::new();
        check_refs(&manifest, dir.path(), true, &mut diags);

        assert!(diags.iter().any(|d| {
            d.level == theta_schema::DiagLevel::Error
                && d.path == "[instructions.rules]"
                && d.message.contains(GIT_UNRESOLVED_CODE)
                && d.message.contains("[instructions.rules.git-rule]")
        }));
        assert!(diags.iter().any(|d| {
            d.level == theta_schema::DiagLevel::Warn
                && d.path == "[skills]"
                && d.message.contains(GIT_UNRESOLVED_CODE)
                && d.message.contains("[skills.git-skill]")
        }));
    }

    #[test]
    fn resolve_content_marks_deferred_and_missing_statuses() {
        let dir = tempfile::tempdir().unwrap();
        fs_err::create_dir_all(dir.path().join("instructions/rules")).unwrap();
        fs_err::write(dir.path().join("instructions/system.md"), "system").unwrap();

        let manifest = manifest_with_all_current_ref_sites();
        let resolved = resolve_content(&manifest, dir.path());

        assert_eq!(
            resolved.get_status(&ResolvedRefKey::instructions_rule("git-rule")),
            Some(theta_schema::ResolutionStatus::Deferred)
        );
        assert_eq!(
            resolved.get_status(&ResolvedRefKey::skill("git-skill")),
            Some(theta_schema::ResolutionStatus::Deferred)
        );
        assert_eq!(
            resolved.get_status(&ResolvedRefKey::instructions_rule("local-rule")),
            Some(theta_schema::ResolutionStatus::Missing)
        );
    }
}
