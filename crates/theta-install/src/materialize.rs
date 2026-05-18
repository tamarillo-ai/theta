//! Materialize locked resources into `.theta/` and clean up orphans.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use theta_lock::{LockFile, LockedSource, ResourceLock};
use theta_static::ThetaProjectLayout as Layout;

#[derive(Debug, thiserror::Error)]
pub enum MaterializeError {
    #[error("could not determine system store directory")]
    StoreDirNotFound,
    #[error("remote source materialization not yet supported for {resource}")]
    UnsupportedRemoteSource { resource: String },
    #[error(
        "{resource} declared in theta.lock but not found at {path} - run `theta lock --force` to re-resolve"
    )]
    SourceNotFound { resource: String, path: PathBuf },
}

#[derive(Debug, Default)]
pub struct SyncReport {
    pub created: usize,
    pub updated: usize,
    pub unchanged: usize,
}

impl SyncReport {
    fn record(&mut self, action: FileAction) {
        match action {
            FileAction::Created => self.created += 1,
            FileAction::Updated => self.updated += 1,
            FileAction::Unchanged => self.unchanged += 1,
        }
    }

    pub fn changed(&self) -> bool {
        self.created > 0 || self.updated > 0
    }
}

enum FileAction {
    Created,
    Updated,
    Unchanged,
}

pub fn materialize(
    lock: &LockFile,
    project_dir: &Path,
    theta_dir: &Path,
    git_cache_dir: &Path,
) -> Result<SyncReport> {
    tracing::info!(theta_dir = %theta_dir.display(), "materializing locked resources");
    let mut report = SyncReport::default();

    materialize_resources(
        lock,
        project_dir,
        theta_dir,
        git_cache_dir,
        false,
        &mut report,
    )?;

    materialize_subagent_resources(lock, project_dir, theta_dir, git_cache_dir, &mut report)?;

    Ok(report)
}

fn materialize_resources(
    lock: &LockFile,
    project_dir: &Path,
    theta_dir: &Path,
    git_cache_dir: &Path,
    skip_subagent_tomls: bool,
    report: &mut SyncReport,
) -> Result<()> {
    if let Some(ref instructions) = lock.instructions {
        if let Some(ref system) = instructions.system {
            report.record(materialize_file(
                project_dir,
                theta_dir,
                system,
                theta_static::SYSTEM_FILE_NAME,
                git_cache_dir,
            )?);
        }

        for (name, entry) in &instructions.rules {
            let action = materialize_file(
                project_dir,
                theta_dir,
                entry,
                &Layout::rule_rel(name),
                git_cache_dir,
            )?;
            report.record(action);
        }
    }

    for (name, entry) in &lock.skills {
        let action = materialize_skill(project_dir, theta_dir, name, entry, git_cache_dir)?;
        report.record(action);
    }

    if !skip_subagent_tomls {
        for (name, entry) in &lock.subagents {
            match entry {
                theta_lock::SubagentLock::Ref { .. } => {
                    let rel = format!(
                        "{}/{}",
                        Layout::subagent_rel(name),
                        theta_static::MANIFEST_FILE_NAME
                    );
                    let resource_lock = entry.as_resource_lock();
                    let action = materialize_file(
                        project_dir,
                        theta_dir,
                        resource_lock,
                        &rel,
                        git_cache_dir,
                    )?;
                    report.record(action);
                }
                theta_lock::SubagentLock::Inline { prompt } => {
                    let rel = theta_static::subagent_system_rel(name);
                    let action =
                        materialize_file(project_dir, theta_dir, prompt, &rel, git_cache_dir)?;
                    report.record(action);
                }
                other => {
                    tracing::warn!(subagent = name, variant = ?other, "unknown SubagentLock variant, skipping materialization");
                }
            }
        }
    }

    Ok(())
}

pub fn cleanup_orphans(lock: &LockFile, theta_dir: &Path) -> Result<()> {
    if !theta_dir.exists() {
        return Ok(());
    }
    tracing::debug!(theta_dir = %theta_dir.display(), "cleaning up orphans");

    let system_path = theta_dir.join(theta_static::SYSTEM_FILE_NAME);
    if system_path.exists() {
        let has_system = lock
            .instructions
            .as_ref()
            .is_some_and(|i| i.system.is_some());
        if !has_system {
            tracing::debug!("removing orphaned system prompt");
            fs_err::remove_file(&system_path)
                .with_context(|| format!("failed to remove orphan {}", system_path.display()))?;
        }
    }

    let rules_dir = theta_dir.join(theta_static::RULES_DIR);
    cleanup_orphan_dir(
        &rules_dir,
        &rules_dir,
        lock.instructions
            .as_ref()
            .map_or(&std::collections::BTreeMap::new(), |i| &i.rules),
    )?;

    let skills_dir = theta_dir.join(theta_static::SKILLS_DIR);
    cleanup_orphan_dir(&skills_dir, &skills_dir, &lock.skills)?;

    let subagent_names: std::collections::BTreeMap<String, &ResourceLock> = lock
        .subagents
        .iter()
        .map(|(k, v)| (k.clone(), v.as_resource_lock()))
        .collect();

    let subagents_dir = theta_dir.join(theta_static::SUBAGENTS_DIR_NAME);
    cleanup_orphan_dir(&subagents_dir, &subagents_dir, &subagent_names)?;

    Ok(())
}

fn materialize_file(
    project_dir: &Path,
    theta_dir: &Path,
    entry: &ResourceLock,
    dest_rel: &str,
    git_cache_dir: &Path,
) -> Result<FileAction> {
    let dest = theta_dir.join(dest_rel);

    let source_path = match &entry.source {
        LockedSource::Path { path } => project_dir.join(path),
        LockedSource::System { system } => {
            let data_dir = theta_dirs::data_dir().ok_or(MaterializeError::StoreDirNotFound)?;
            let store = theta_static::SystemStoreLayout::new(&data_dir);
            store.rule(system)
        }
        LockedSource::Git {
            git,
            resolved_commit,
            file,
            ..
        } => {
            let fetcher = theta_git::GitFetcher::new(git_cache_dir.to_path_buf());
            let result = fetcher.fetch(
                git,
                &theta_git::GitRef::Commit(resolved_commit.clone()),
                Some(resolved_commit),
            )?;
            match file {
                Some(f) => result.path.join(f),
                None => result.path.clone(),
            }
        }
        _ => {
            return Err(MaterializeError::UnsupportedRemoteSource {
                resource: dest_rel.to_string(),
            }
            .into());
        }
    };

    if !source_path.exists() {
        return Err(MaterializeError::SourceNotFound {
            resource: dest_rel.to_string(),
            path: source_path.clone(),
        }
        .into());
    }

    tracing::debug!(dest_rel, source = %source_path.display(), "materializing file");
    let already_existed = dest.exists();

    if already_existed {
        let existing =
            fs_err::read(&dest).with_context(|| format!("failed to read {}", dest.display()))?;
        if theta_lock::content_hash(&existing) == entry.content_hash {
            return Ok(FileAction::Unchanged);
        }
    }

    if let Some(parent) = dest.parent() {
        fs_err::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let content = fs_err::read(&source_path)
        .with_context(|| format!("failed to read {}", source_path.display()))?;
    let parent = dest.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file in {}", parent.display()))?;
    tmp.write_all(&content)
        .with_context(|| format!("failed to write temp file for {}", dest.display()))?;
    tmp.persist(&dest)
        .map_err(|e| anyhow::anyhow!("failed to persist temp file to {}: {}", dest.display(), e))?;

    let action = if already_existed {
        tracing::debug!(dest_rel, "updated");
        FileAction::Updated
    } else {
        tracing::debug!(dest_rel, "created");
        FileAction::Created
    };
    Ok(action)
}

fn materialize_skill(
    project_dir: &Path,
    theta_dir: &Path,
    name: &str,
    entry: &ResourceLock,
    git_cache_dir: &Path,
) -> Result<FileAction> {
    let dest_dir = theta_dir.join(theta_static::SKILLS_DIR).join(name);

    let source_dir = match &entry.source {
        LockedSource::Path { path } => project_dir.join(path),
        LockedSource::System { system } => {
            let data_dir = theta_dirs::data_dir().ok_or(MaterializeError::StoreDirNotFound)?;
            let store = theta_static::SystemStoreLayout::new(&data_dir);
            store.skill(system)
        }
        LockedSource::Git {
            git,
            resolved_commit,
            subdirectory,
            ..
        } => {
            let fetcher = theta_git::GitFetcher::new(git_cache_dir.to_path_buf());
            let result = fetcher.fetch(
                git,
                &theta_git::GitRef::Commit(resolved_commit.clone()),
                Some(resolved_commit),
            )?;
            match subdirectory.as_deref() {
                Some(sub) => result.path.join(sub),
                None => result.path.clone(),
            }
        }
        _ => {
            return Err(MaterializeError::UnsupportedRemoteSource {
                resource: format!("skills.{name}"),
            }
            .into());
        }
    };

    if !source_dir.exists() {
        return Err(MaterializeError::SourceNotFound {
            resource: format!("skills.{name}"),
            path: source_dir.clone(),
        }
        .into());
    }

    tracing::debug!(skill = name, source = %source_dir.display(), "materializing skill");
    let already_existed = dest_dir.exists();

    if already_existed {
        match theta_lock::skill_content_hash(&dest_dir) {
            Ok(existing_hash) if existing_hash == entry.content_hash => {
                return Ok(FileAction::Unchanged);
            }
            _ => {} // hash mismatch, missing files, or error --> re-copy
        }
    }

    theta_static::copy_dir_recursive(&source_dir, &dest_dir)
        .with_context(|| format!("failed to copy skill \"{name}\""))?;

    Ok(if already_existed {
        FileAction::Updated
    } else {
        FileAction::Created
    })
}

fn materialize_subagent_resources(
    lock: &LockFile,
    project_dir: &Path,
    theta_dir: &Path,
    git_cache_dir: &Path,
    report: &mut SyncReport,
) -> Result<()> {
    for (name, sub_lock) in &lock.subagents {
        match sub_lock {
            theta_lock::SubagentLock::Ref {
                resource,
                instructions,
                skills,
                ..
            } => {
                let subagent_theta_dir =
                    theta_dir.join(theta_static::SUBAGENTS_DIR_NAME).join(name);
                let child_project_dir = match &resource.source {
                    LockedSource::Path { path } => {
                        let manifest_path = project_dir.join(path);
                        manifest_path.parent().unwrap_or(project_dir).to_path_buf()
                    }
                    _ => project_dir.to_path_buf(),
                };

                let child_lock = LockFile::new(
                    lock.meta.clone(),
                    instructions.clone(),
                    skills.clone(),
                    std::collections::BTreeMap::new(),
                );

                materialize_resources(
                    &child_lock,
                    &child_project_dir,
                    &subagent_theta_dir,
                    git_cache_dir,
                    true,
                    report,
                )?;
            }
            theta_lock::SubagentLock::Inline { .. } => {
                // inline subagent resources (system.md) are already materialized
                // in the main materialize_resources loop above
            }
            other => {
                tracing::warn!(subagent = name, variant = ?other, "unknown SubagentLock variant, skipping resource materialization");
            }
        }
    }

    Ok(())
}

/// Remove files/dirs under `dir` not in `locked_entries`.
/// `base` is the top-level dir (e.g. `.theta/rules/`) used to compute
/// relative keys for path-qualified names like `"review/pr-review"`.
fn cleanup_orphan_dir<V>(
    dir: &Path,
    base: &Path,
    locked_entries: &std::collections::BTreeMap<String, V>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in
        fs_err::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let rel_os = path.strip_prefix(base).unwrap_or(&path);
        let Some(rel) = rel_os.to_str() else {
            tracing::warn!(path = %path.display(), "skipping non-UTF-8 entry in .theta/");
            continue;
        };
        let ft = entry.file_type()?;

        if ft.is_symlink() {
            if !locked_entries.contains_key(rel) {
                fs_err::remove_file(&path).with_context(|| {
                    format!("failed to remove orphan symlink {}", path.display())
                })?;
            }
        } else if ft.is_dir() {
            let prefix = format!("{rel}/");
            let has_locked_child = locked_entries.keys().any(|k| k.starts_with(&prefix));
            if has_locked_child {
                cleanup_orphan_dir(&path, base, locked_entries)?;
                // if recursion emptied it, remove the dir
                if path.is_dir() && fs_err::read_dir(&path)?.next().is_none() {
                    fs_err::remove_dir(&path).with_context(|| {
                        format!("failed to remove empty directory {}", path.display())
                    })?;
                }
            } else if locked_entries.contains_key(rel) {
                // flat key matches a directory name - stale dir from a
                // previous layout that used path-qualified names
                tracing::warn!(
                    "directory {} shadows flat locked entry \"{}\"; \
                     remove the directory manually if it is stale",
                    path.display(),
                    rel,
                );
            } else {
                fs_err::remove_dir_all(&path).with_context(|| {
                    format!("failed to remove orphan directory {}", path.display())
                })?;
            }
        } else {
            let key = rel.strip_suffix(".md").unwrap_or(rel);
            if !locked_entries.contains_key(key) {
                fs_err::remove_file(&path)
                    .with_context(|| format!("failed to remove orphan file {}", path.display()))?;
            }
        }
    }

    Ok(())
}
