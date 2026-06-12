//! `theta sync` — materialize dependencies into `.theta/`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{OutputFormat, SyncArgs};
use theta_lock::{LockedSource, read_lock};
use theta_manifest::read_manifest;
use theta_static::DOT_THETA_DIR;
use theta_static::LOCKFILE;

use crate::resolve::validate_materialized;

use super::output::{present, present_error, present_no_op};
use super::{project_dir, require_manifest};

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub(crate) struct SyncOutcome {
    pub theta_dir: PathBuf,
    pub created: usize,
    pub updated: usize,
}

pub(crate) fn execute(
    args: SyncArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    require_manifest(manifest_path)?;
    let json = matches!(output_format, OutputFormat::Json);

    if args.force {
        super::lock::execute(
            theta_args::LockArgs { force: true },
            OutputFormat::Human,
            manifest_path,
        )?;
    } else {
        super::lock::ensure_locked(manifest_path)?;
    }

    let project_dir = project_dir(manifest_path)?;
    let out_dir = std::env::var(theta_static::THETA_OUT_DIR_ENV)
        .ok()
        .map_or_else(|| project_dir.to_path_buf(), PathBuf::from);
    let lock_path = out_dir.join(LOCKFILE);
    let theta_dir = out_dir.join(DOT_THETA_DIR);

    let lock =
        read_lock(&lock_path).with_context(|| format!("failed to read {}", lock_path.display()))?;

    let remote_count = count_remote_sources(&lock);
    let pb = if remote_count > 0 && !json {
        let pb = ProgressBar::new(remote_count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {spinner:.green} [{bar:30}] {pos}/{len} {msg}")
                .expect("valid template")
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("resolving remote sources...");
        Some(pb)
    } else {
        None
    };

    let git_cache = theta_git::cache_dir()?;
    let report = theta_install::materialize(&lock, project_dir, &theta_dir, &git_cache)?;
    theta_install::cleanup_orphans(&lock, &theta_dir)?;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {} for validation", manifest_path.display()))?;
    let mut diags = Vec::new();
    validate_materialized(&manifest, project_dir, &mut diags);

    let errors = diags
        .iter()
        .filter(|d| matches!(d.level, theta_schema::DiagLevel::Error))
        .count();

    let outcome = SyncOutcome {
        theta_dir,
        created: report.created,
        updated: report.updated,
    };

    if errors > 0 {
        let warnings = diags.len() - errors;
        let err = anyhow!(
            ".theta/ materialized but content validation failed: {errors} error(s), {warnings} warning(s)"
        );
        let diags_for_render = diags.clone();
        return present_error(
            &["sync"],
            output_format,
            outcome,
            diags,
            move |_| {
                super::report_diagnostics(&diags_for_render);
            },
            err,
        );
    }

    if report.changed() {
        present(&["sync"], output_format, outcome, diags, |o| {
            anstream::eprintln!(
                "{} .theta/ materialized ({} created, {} updated)",
                "synced".green().bold(),
                o.created,
                o.updated,
            );
        })
    } else {
        present_no_op(&["sync"], output_format, outcome, diags, |_| {
            anstream::eprintln!("{} .theta/ is up to date", "synced".green().bold());
        })
    }
}

fn count_remote_sources(lock: &theta_lock::LockFile) -> usize {
    let is_remote = |src: &LockedSource| matches!(src, LockedSource::Git { .. });

    let mut count = 0;
    if let Some(ref instr) = lock.instructions {
        count += usize::from(
            instr
                .system
                .as_ref()
                .filter(|s| is_remote(&s.source))
                .is_some(),
        );
        count += instr
            .rules
            .values()
            .filter(|e| is_remote(&e.source))
            .count();
    }
    count += lock
        .skills
        .values()
        .filter(|e| is_remote(&e.source))
        .count();
    count
}
