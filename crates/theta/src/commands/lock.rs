//! `theta lock` — resolve all sources and write `theta.lock`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{LockArgs, OutputFormat};
use theta_git::cache_dir;
use theta_lock::{build_lock, is_stale, read_lock, write_lock};
use theta_manifest::read_manifest;
use theta_schema::Diagnostic;
use theta_static::LOCKFILE;

use super::output::{present, present_error, present_no_op};
use super::{project_dir, require_manifest};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct LockOutcome {
    pub lockfile_path: PathBuf,
    pub wrote: bool,
}

pub(crate) fn execute(
    args: LockArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    require_manifest(manifest_path)?;

    let project_dir = project_dir(manifest_path)?;
    let out_dir = std::env::var(theta_static::THETA_OUT_DIR_ENV)
        .ok()
        .map_or_else(|| project_dir.to_path_buf(), std::path::PathBuf::from);
    let lock_path = out_dir.join(LOCKFILE);

    let manifest_bytes = fs_err::read(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if !args.force
        && lock_path.exists()
        && let Ok(existing) = read_lock(&lock_path)
        && is_stale(&existing, &manifest_bytes, project_dir)?.is_none()
    {
        let outcome = LockOutcome {
            lockfile_path: lock_path,
            wrote: false,
        };
        return present_no_op(&["lock"], output_format, outcome, vec![], |_| {
            anstream::eprintln!("{} theta.lock is up to date", "ok".green().bold());
        });
    }

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let cache = cache_dir()?;
    let lock = match build_lock(&manifest, &manifest_bytes, project_dir, &cache) {
        Ok(l) => l,
        Err(errors) => {
            let diags: Vec<Diagnostic> = errors
                .iter()
                .map(|e| Diagnostic::error("[lock]", e.to_string()))
                .collect();
            let outcome = LockOutcome {
                lockfile_path: lock_path,
                wrote: false,
            };
            let n = errors.len();
            return present_error(
                &["lock"],
                output_format,
                outcome,
                diags,
                |_| {
                    for e in &errors {
                        anstream::eprintln!("{} {}", "error".red().bold(), e);
                    }
                },
                anyhow!("failed to lock: {n} error(s) - all declared sources must be reachable"),
            );
        }
    };

    if let Some(parent) = lock_path.parent() {
        fs_err::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    write_lock(&lock_path, &lock)
        .with_context(|| format!("failed to write {}", lock_path.display()))?;

    let outcome = LockOutcome {
        lockfile_path: lock_path,
        wrote: true,
    };
    present(&["lock"], output_format, outcome, vec![], |_| {
        anstream::eprintln!("{} wrote {}", "locked".green().bold(), LOCKFILE.cyan());
    })
}

/// Lock if needed (lock missing or stale). Used by sync and cast.
pub(crate) fn ensure_locked(manifest_path: &Path) -> Result<()> {
    let project_dir = project_dir(manifest_path)?;
    let out_dir = std::env::var(theta_static::THETA_OUT_DIR_ENV)
        .ok()
        .map_or_else(|| project_dir.to_path_buf(), std::path::PathBuf::from);
    let lock_path = out_dir.join(LOCKFILE);

    let manifest_bytes = fs_err::read(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let needs_lock = if lock_path.exists() {
        match read_lock(&lock_path) {
            Ok(existing) => is_stale(&existing, &manifest_bytes, project_dir)?.is_some(),
            Err(e) => {
                tracing::debug!(path = %lock_path.display(), error = %e, "corrupt or unreadable lock file, treating as stale");
                true
            }
        }
    } else {
        true
    };

    if needs_lock {
        let manifest = read_manifest(manifest_path)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

        let cache = cache_dir()?;
        let lock =
            build_lock(&manifest, &manifest_bytes, project_dir, &cache).map_err(|errors| {
                for e in &errors {
                    anstream::eprintln!("{} {}", "error".red().bold(), e);
                }
                anyhow::anyhow!(
                    "failed to lock: {} error(s) - all declared sources must be reachable",
                    errors.len()
                )
            })?;

        if let Some(parent) = lock_path.parent() {
            fs_err::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        write_lock(&lock_path, &lock)
            .with_context(|| format!("failed to write {}", lock_path.display()))?;

        anstream::eprintln!("{} wrote {}", "locked".green().bold(), LOCKFILE.cyan());
    }

    Ok(())
}
