use std::collections::BTreeMap;
use std::path::Path;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::error::BuildError;
use crate::hash::{content_hash, manifest_hash, skill_content_hash};
use crate::types::{
    InstructionsLock, LockFile, LockMeta, LockedSource, ResourceLock, SubagentLock,
};

/// Resolve every source declared in a manifest and produce a `LockFile`.
/// Returns the list of errors if any source is unreachable.
pub fn build_lock(
    manifest: &theta_schema::ThetaManifest,
    manifest_bytes: &[u8],
    project_dir: &Path,
    git_cache_dir: &Path,
) -> Result<LockFile, Vec<BuildError>> {
    let mut errors: Vec<BuildError> = Vec::new();

    let mhash = match manifest_hash(manifest_bytes) {
        Ok(h) => h,
        Err(e) => {
            return Err(vec![e.into()]);
        }
    };

    let instructions_lock =
        build_instructions_lock(manifest, project_dir, git_cache_dir, &mut errors);
    let skills = build_skills_lock(manifest, project_dir, git_cache_dir, &mut errors);
    let subagents = build_subagents_lock(manifest, project_dir, git_cache_dir, &mut errors);

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(LockFile {
        meta: LockMeta {
            schema: manifest.theta.schema.clone(),
            manifest_hash: mhash,
        },
        instructions: instructions_lock,
        skills,
        subagents,
    })
}

fn build_instructions_lock(
    manifest: &theta_schema::ThetaManifest,
    project_dir: &Path,
    git_cache_dir: &Path,
    errors: &mut Vec<BuildError>,
) -> Option<InstructionsLock> {
    let instructions = manifest.instructions.as_ref()?;

    let system = instructions.system.as_ref().and_then(|sys_ref| {
        let path_str = sys_ref.as_str();
        let full = project_dir.join(path_str);
        match fs_err::read(&full) {
            Ok(data) => Some(ResourceLock {
                source: LockedSource::Path {
                    path: path_str.to_string(),
                },
                content_hash: content_hash(&data),
            }),
            Err(e) => {
                errors.push(BuildError::SourceNotFound {
                    resource: "instructions.system".into(),
                    path: full,
                    source: e,
                });
                None
            }
        }
    });

    let rules = if let Some(ref rule_map) = instructions.rules {
        let results: Vec<Result<(String, ResourceLock), BuildError>> = rule_map
            .par_iter()
            .map(|(name, rule)| resolve_one_rule(name, rule, project_dir, git_cache_dir))
            .collect();

        let mut map = BTreeMap::new();
        for result in results {
            match result {
                Ok((name, lock)) => {
                    map.insert(name, lock);
                }
                Err(e) => errors.push(e),
            }
        }
        map
    } else {
        BTreeMap::new()
    };

    if system.is_none() && rules.is_empty() {
        return None;
    }

    Some(InstructionsLock { system, rules })
}

fn resolve_one_rule(
    name: &str,
    rule: &theta_schema::Rule,
    project_dir: &Path,
    git_cache_dir: &Path,
) -> Result<(String, ResourceLock), BuildError> {
    let resource = format!("instructions.rules.{name}");
    match &rule.src {
        theta_schema::LocalOrGitRef::Local(path_ref) => {
            resolve_local_rule(&resource, name, path_ref.as_str(), project_dir)
        }
        theta_schema::LocalOrGitRef::Git {
            git,
            branch,
            tag,
            rev,
            file,
        } => {
            let reference =
                theta_git::GitRef::from_manifest(branch.as_deref(), tag.as_deref(), rev.as_deref());
            resolve_git_rule(&resource, name, git, &reference, file, git_cache_dir)
        }
        theta_schema::LocalOrGitRef::System { system } => {
            resolve_system_rule(&resource, name, system)
        }
        _ => Err(BuildError::UnsupportedSource { resource }),
    }
}

fn resolve_local_rule(
    resource: &str,
    name: &str,
    path_str: &str,
    project_dir: &Path,
) -> Result<(String, ResourceLock), BuildError> {
    let full = project_dir.join(path_str);
    let data = fs_err::read(&full).map_err(|e| BuildError::SourceNotFound {
        resource: resource.to_string(),
        path: full,
        source: e,
    })?;
    Ok((
        name.to_string(),
        ResourceLock {
            source: LockedSource::Path {
                path: path_str.to_string(),
            },
            content_hash: content_hash(&data),
        },
    ))
}

fn resolve_system_rule(
    resource: &str,
    name: &str,
    system: &str,
) -> Result<(String, ResourceLock), BuildError> {
    let data_dir = theta_dirs::data_dir().ok_or_else(|| BuildError::NoDataDir {
        resource: resource.to_string(),
    })?;
    let store = theta_static::SystemStoreLayout::new(&data_dir);
    let rule_path = store.rule(system);
    if !rule_path.exists() {
        return Err(BuildError::SystemStoreNotFound {
            resource: resource.to_string(),
            kind: "rule",
            name: system.to_string(),
        });
    }
    let data = fs_err::read(&rule_path).map_err(|e| BuildError::SourceNotFound {
        resource: resource.to_string(),
        path: rule_path,
        source: e,
    })?;
    Ok((
        name.to_string(),
        ResourceLock {
            source: LockedSource::System {
                system: system.to_string(),
            },
            content_hash: content_hash(&data),
        },
    ))
}

fn build_skills_lock(
    manifest: &theta_schema::ThetaManifest,
    project_dir: &Path,
    git_cache_dir: &Path,
    errors: &mut Vec<BuildError>,
) -> BTreeMap<String, ResourceLock> {
    let Some(ref skills) = manifest.skills else {
        return BTreeMap::new();
    };

    let results: Vec<Result<(String, ResourceLock), BuildError>> = skills
        .par_iter()
        .map(|(name, skill)| resolve_one_skill(name, skill, project_dir, git_cache_dir))
        .collect();

    let mut out = BTreeMap::new();
    for result in results {
        match result {
            Ok((name, lock)) => {
                out.insert(name, lock);
            }
            Err(e) => errors.push(e),
        }
    }
    out
}

fn resolve_one_skill(
    name: &str,
    skill: &theta_schema::Skill,
    project_dir: &Path,
    git_cache_dir: &Path,
) -> Result<(String, ResourceLock), BuildError> {
    let resource = format!("skills.{name}");
    match &skill.source {
        theta_schema::SourceRef::Path { path } => {
            resolve_local_skill(&resource, name, path, project_dir)
        }
        theta_schema::SourceRef::Git {
            git,
            branch,
            tag,
            rev,
            subdirectory,
        } => {
            let reference =
                theta_git::GitRef::from_manifest(branch.as_deref(), tag.as_deref(), rev.as_deref());
            resolve_git_skill(
                &resource,
                name,
                git,
                &reference,
                subdirectory.as_deref(),
                git_cache_dir,
            )
        }
        theta_schema::SourceRef::System { system } => resolve_system_skill(&resource, name, system),
        _ => Err(BuildError::UnsupportedSource { resource }),
    }
}

fn resolve_local_skill(
    resource: &str,
    name: &str,
    path: &str,
    project_dir: &Path,
) -> Result<(String, ResourceLock), BuildError> {
    let full = project_dir.join(path);
    let hash = skill_content_hash(&full).map_err(|e| BuildError::SourceNotFound {
        resource: resource.to_string(),
        path: full.join(theta_static::SKILL_FILE_NAME),
        source: e,
    })?;
    Ok((
        name.to_string(),
        ResourceLock {
            source: LockedSource::Path {
                path: path.to_string(),
            },
            content_hash: hash,
        },
    ))
}

fn resolve_system_skill(
    resource: &str,
    name: &str,
    system: &str,
) -> Result<(String, ResourceLock), BuildError> {
    let data_dir = theta_dirs::data_dir().ok_or_else(|| BuildError::NoDataDir {
        resource: resource.to_string(),
    })?;
    let store = theta_static::SystemStoreLayout::new(&data_dir);
    let skill_dir = store.skill(system);
    if !skill_dir.exists() {
        return Err(BuildError::SystemStoreNotFound {
            resource: resource.to_string(),
            kind: "skill",
            name: system.to_string(),
        });
    }
    let hash = skill_content_hash(&skill_dir).map_err(|e| BuildError::HashFailed {
        resource: resource.to_string(),
        source: e,
    })?;
    Ok((
        name.to_string(),
        ResourceLock {
            source: LockedSource::System {
                system: system.to_string(),
            },
            content_hash: hash,
        },
    ))
}

fn resolve_git_skill(
    resource: &str,
    name: &str,
    git_url: &str,
    reference: &theta_git::GitRef,
    subdirectory: Option<&str>,
    git_cache_dir: &Path,
) -> Result<(String, ResourceLock), BuildError> {
    let fetcher = theta_git::GitFetcher::new(git_cache_dir.to_path_buf());
    let result = fetcher
        .fetch(git_url, reference, None)
        .map_err(|e| BuildError::GitFetch {
            resource: resource.to_string(),
            source: e,
        })?;
    let skill_dir = match subdirectory {
        Some(sub) => result.path.join(sub),
        None => result.path.clone(),
    };
    let skill_md = skill_dir.join(theta_static::SKILL_FILE_NAME);
    if !skill_md.exists() {
        return Err(BuildError::GitFileNotFound {
            resource: resource.to_string(),
            url: git_url.to_string(),
            commit: result.commit.short().to_string(),
            file: format!(
                "{}/{}",
                subdirectory.unwrap_or("<repo root>"),
                theta_static::SKILL_FILE_NAME
            ),
        });
    }
    let hash = skill_content_hash(&skill_dir).map_err(|e| BuildError::HashFailed {
        resource: resource.to_string(),
        source: e,
    })?;
    Ok((
        name.to_string(),
        ResourceLock {
            source: LockedSource::Git {
                git: git_url.to_string(),
                git_ref: reference.to_string(),
                resolved_commit: result.commit.clone(),
                subdirectory: subdirectory.map(std::string::ToString::to_string),
                file: None,
            },
            content_hash: hash,
        },
    ))
}

fn resolve_git_rule(
    resource: &str,
    name: &str,
    git_url: &str,
    reference: &theta_git::GitRef,
    file: &str,
    git_cache_dir: &Path,
) -> Result<(String, ResourceLock), BuildError> {
    let fetcher = theta_git::GitFetcher::new(git_cache_dir.to_path_buf());
    let result = fetcher
        .fetch(git_url, reference, None)
        .map_err(|e| BuildError::GitFetch {
            resource: resource.to_string(),
            source: e,
        })?;
    let file_path = result.path.join(file);
    let data = fs_err::read(&file_path).map_err(|_| BuildError::GitFileNotFound {
        resource: resource.to_string(),
        url: git_url.to_string(),
        commit: result.commit.short().to_string(),
        file: file.to_string(),
    })?;
    Ok((
        name.to_string(),
        ResourceLock {
            source: LockedSource::Git {
                git: git_url.to_string(),
                git_ref: reference.to_string(),
                resolved_commit: result.commit.clone(),
                subdirectory: None,
                file: Some(file.to_string()),
            },
            content_hash: content_hash(&data),
        },
    ))
}

fn build_subagents_lock(
    manifest: &theta_schema::ThetaManifest,
    project_dir: &Path,
    git_cache_dir: &Path,
    errors: &mut Vec<BuildError>,
) -> BTreeMap<String, SubagentLock> {
    let mut out = BTreeMap::new();
    let Some(ref subagents) = manifest.subagents else {
        return out;
    };
    for sub in subagents {
        if let Some(ref agent_ref) = sub.agent_ref {
            lock_ref_subagent(
                &mut out,
                sub,
                agent_ref.as_str(),
                project_dir,
                git_cache_dir,
                errors,
            );
        } else if let Some(ref prompt_path) = sub.prompt_path {
            lock_inline_subagent(&mut out, sub, prompt_path, project_dir, errors);
        }
        // description-only: no lock entry
    }
    out
}

fn lock_ref_subagent(
    out: &mut BTreeMap<String, SubagentLock>,
    sub: &theta_schema::Subagent,
    agent_ref: &str,
    project_dir: &Path,
    git_cache_dir: &Path,
    errors: &mut Vec<BuildError>,
) {
    let resource = format!("subagents.{}", sub.name);
    let full = project_dir.join(agent_ref);
    match fs_err::read(&full) {
        Ok(data) => {
            let child_hash = content_hash(&data);
            let child_project_dir = full.parent().unwrap_or(project_dir);
            match toml::from_str::<theta_schema::ThetaManifest>(&String::from_utf8_lossy(&data)) {
                Ok(child_manifest) => {
                    let child_instructions = build_instructions_lock(
                        &child_manifest,
                        child_project_dir,
                        git_cache_dir,
                        errors,
                    );
                    let child_skills = build_skills_lock(
                        &child_manifest,
                        child_project_dir,
                        git_cache_dir,
                        errors,
                    );
                    out.insert(
                        sub.name.clone(),
                        SubagentLock::Ref {
                            resource: ResourceLock {
                                source: LockedSource::Path {
                                    path: agent_ref.to_string(),
                                },
                                content_hash: child_hash,
                            },
                            instructions: child_instructions,
                            skills: child_skills,
                        },
                    );
                }
                Err(e) => {
                    errors.push(BuildError::ChildManifestParse {
                        resource,
                        source: e,
                    });
                }
            }
        }
        Err(e) => {
            errors.push(BuildError::SourceNotFound {
                resource,
                path: full,
                source: e,
            });
        }
    }
}

fn lock_inline_subagent(
    out: &mut BTreeMap<String, SubagentLock>,
    sub: &theta_schema::Subagent,
    prompt_path: &str,
    project_dir: &Path,
    errors: &mut Vec<BuildError>,
) {
    let full = project_dir.join(prompt_path);
    match fs_err::read(&full) {
        Ok(data) => {
            out.insert(
                sub.name.clone(),
                SubagentLock::Inline {
                    prompt: ResourceLock {
                        source: LockedSource::Path {
                            path: prompt_path.to_string(),
                        },
                        content_hash: content_hash(&data),
                    },
                },
            );
        }
        Err(e) => {
            errors.push(BuildError::SourceNotFound {
                resource: format!("subagents.{}", sub.name),
                path: full,
                source: e,
            });
        }
    }
}
