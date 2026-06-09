//! `theta lock` — resolve all sources and write `theta.lock`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{LockArgs, OutputFormat};
use theta_git::cache_dir;
use theta_lock::{build_lock, is_stale, read_lock, write_lock};
use theta_manifest::read_manifest;
use theta_schema::{CommandFailure, CommandOutput, Diagnostic};
use theta_static::LOCKFILE;

use super::{project_dir, require_manifest};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct LockOutput {
    pub lockfile_path: PathBuf,
    pub wrote: bool,
}

pub(crate) fn execute(
    args: LockArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

    let project_dir = project_dir(manifest_path)?;
    let lock_path = project_dir.join(LOCKFILE);

    let manifest_bytes = fs_err::read(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    if !args.force && lock_path.exists() {
        if let Ok(existing) = read_lock(&lock_path) {
            if is_stale(&existing, &manifest_bytes, project_dir)?.is_none() {
                if json {
                    CommandOutput::no_op(
                        ["lock"],
                        LockOutput {
                            lockfile_path: lock_path,
                            wrote: false,
                        },
                    )
                    .print_json()?;
                } else {
                    anstream::eprintln!("{} theta.lock is up to date", "ok".green().bold());
                }
                return Ok(());
            }
        }
    }

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let cache = cache_dir()?;
    let lock = match build_lock(&manifest, &manifest_bytes, project_dir, &cache) {
        Ok(l) => l,
        Err(errors) => {
            if json {
                let diags: Vec<Diagnostic> = errors
                    .iter()
                    .map(|e| Diagnostic::error("[lock]", e.to_string()))
                    .collect();
                CommandOutput::error(
                    ["lock"],
                    LockOutput {
                        lockfile_path: lock_path,
                        wrote: false,
                    },
                    diags,
                )
                .print_json()?;
                return Err(CommandFailure.into());
            }
            for e in &errors {
                anstream::eprintln!("{} {}", "error".red().bold(), e);
            }
            anyhow::bail!(
                "failed to lock: {} error(s) - all declared sources must be reachable",
                errors.len()
            );
        }
    };

    write_lock(&lock_path, &lock)
        .with_context(|| format!("failed to write {}", lock_path.display()))?;

    if json {
        CommandOutput::ok(
            ["lock"],
            LockOutput {
                lockfile_path: lock_path,
                wrote: true,
            },
        )
        .print_json()?;
    } else {
        anstream::eprintln!("{} wrote {}", "locked".green().bold(), LOCKFILE.cyan());
    }
    Ok(())
}

/// Lock if needed (lock missing or stale). Used by sync and cast.
pub(crate) fn ensure_locked(manifest_path: &Path) -> Result<()> {
    let project_dir = project_dir(manifest_path)?;
    let lock_path = project_dir.join(LOCKFILE);

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

        write_lock(&lock_path, &lock)
            .with_context(|| format!("failed to write {}", lock_path.display()))?;

        anstream::eprintln!("{} wrote {}", "locked".green().bold(), LOCKFILE.cyan());
    }

    Ok(())
}
