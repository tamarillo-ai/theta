//! Cast `theta.toml` to harness-native config formats.
//!
//! One caster per harness; receives the full manifest and decides how to map fields.
//! Casters are pure functions — they return `(relative_path, content)` pairs and
//! leave file writes to the caller.
//!
//! After producing files, `validate_output` checks harness-specific non-functional
//! requirements (size limits, line counts) and returns diagnostics.

mod common;
pub mod harness_config;
mod harnesses;

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use theta_harness::HarnessTarget;
use theta_schema::{Diagnostic, ThetaManifest};

pub use common::{CastContent, CastFile};
pub use harnesses::{ClaudeCode, CodexCli, Copilot, CursorHarness};

pub trait Caster {
    /// Build the set of files to write, from the manifest alone.
    fn cast_files(&self, manifest: &ThetaManifest, theta_dir: &Path) -> Result<Vec<CastFile>>;

    /// Build the set of files to write, merging with existing on-disk files
    /// at `output_dir` when the harness owns configuration files that MAY contain
    /// keys outside of theta's scope.
    fn cast_files_with_output(
        &self,
        manifest: &ThetaManifest,
        theta_dir: &Path,
        _output_dir: &Path,
    ) -> Result<Vec<CastFile>> {
        self.cast_files(manifest, theta_dir)
    }

    fn validate_output(&self, _files: &[CastFile]) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn validate_config(&self, _manifest: &ThetaManifest) -> Vec<Diagnostic> {
        Vec::new()
    }
}

/// Options passed to importers from the CLI.
pub struct ImportOptions {
    /// Directory to write externalized subagent prompt files.
    /// Defaults to `<project>/subagents/`.
    pub subagent_prompts_dir: PathBuf,
    /// Overwrite existing subagent prompt files that have different content.
    pub force_overwrite: bool,
    /// Also import files from other harness locations the source harness discovers.
    pub cross_read: bool,
}

impl ImportOptions {
    pub fn new(
        project_dir: &Path,
        subagent_prompts: Option<PathBuf>,
        force_overwrite: bool,
    ) -> Self {
        let dir = subagent_prompts
            .or_else(|| {
                std::env::var(theta_static::THETA_SUBAGENTS_DIR_ENV)
                    .ok()
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| project_dir.join(theta_static::SUBAGENTS_DIR_NAME));
        Self {
            subagent_prompts_dir: dir,
            force_overwrite,
            cross_read: false,
        }
    }

    #[must_use]
    pub fn with_cross_read(mut self, cross_read: bool) -> Self {
        self.cross_read = cross_read;
        self
    }

    /// Default options for the given project dir — writes to `<project>/subagents/`.
    pub fn default_for(project_dir: &Path) -> Self {
        Self::new(project_dir, None, false)
    }
}

pub trait Importer {
    fn import(&self, project_dir: &Path, opts: &ImportOptions) -> Result<ImportResult>;
}

pub struct ImportResult {
    pub document: toml_edit::DocumentMut,
    pub extracted_files: Vec<CastFile>,
    pub sources_read: Vec<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn caster_for(target: HarnessTarget) -> &'static dyn Caster {
    match target {
        HarnessTarget::ClaudeCode => &harnesses::ClaudeCode,
        HarnessTarget::CodexCli => &harnesses::CodexCli,
        HarnessTarget::Copilot => &harnesses::Copilot,
        HarnessTarget::Cursor => &harnesses::CursorHarness,
        _ => panic!("no caster for harness target: {target}"),
    }
}

pub fn importer_for(target: HarnessTarget) -> &'static dyn Importer {
    match target {
        HarnessTarget::ClaudeCode => &harnesses::ClaudeCode,
        HarnessTarget::CodexCli => &harnesses::CodexCli,
        HarnessTarget::Copilot => &harnesses::Copilot,
        HarnessTarget::Cursor => &harnesses::CursorHarness,
        _ => panic!("no importer for harness target: {target}"),
    }
}

pub fn cast_notes(target: HarnessTarget) -> String {
    match target {
        HarnessTarget::ClaudeCode => harnesses::claude_code_cast_notes(),
        HarnessTarget::CodexCli => harnesses::codex_cli_cast_notes(),
        HarnessTarget::Copilot => harnesses::copilot_cast_notes(),
        HarnessTarget::Cursor => harnesses::cursor_cast_notes(),
        _ => "no known limitations documented for this harness.\n".to_string(),
    }
}

pub fn import_notes(target: HarnessTarget) -> String {
    match target {
        HarnessTarget::ClaudeCode => harnesses::claude_code_import_notes(),
        HarnessTarget::CodexCli => harnesses::codex_cli_import_notes(),
        HarnessTarget::Copilot => harnesses::copilot_import_notes(),
        HarnessTarget::Cursor => harnesses::cursor_import_notes(),
        _ => "no known limitations documented for this harness.\n".to_string(),
    }
}

pub fn write_cast_output(files: &[CastFile], output_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut seen = std::collections::HashSet::new();
    let deduped: Vec<&CastFile> = files
        .iter()
        .filter(|(path, _)| {
            if seen.insert(path.clone()) {
                true
            } else {
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "warn: duplicate output path {} - keeping the first entry, skipping this one",
                    path.display(),
                );
                false
            }
        })
        .collect();

    let parent = match output_dir.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let staging = tempfile::tempdir_in(parent)
        .with_context(|| format!("failed to create staging dir in {}", parent.display()))?;

    for (rel_path, content) in &deduped {
        let staged = staging.path().join(rel_path);
        if let Some(p) = staged.parent() {
            fs_err::create_dir_all(p)
                .with_context(|| format!("failed to create {}", p.display()))?;
        }
        let bytes = match content {
            common::CastContent::Text(s) => {
                if s.ends_with('\n') {
                    s.as_bytes().to_vec()
                } else {
                    format!("{s}\n").into_bytes()
                }
            }
            common::CastContent::Binary(b) => b.clone(),
        };
        fs_err::write(&staged, bytes)
            .with_context(|| format!("failed to write {}", staged.display()))?;
    }

    // backup existing files before overwriting so rollback can restore them
    let backup_dir = staging.path().join(".backups");
    let mut written: Vec<PathBuf> = Vec::new();
    let mut backed_up: Vec<(PathBuf, PathBuf)> = Vec::new(); // (backup, original)

    let commit_result = (|| -> Result<()> {
        for (rel_path, _) in &deduped {
            let src = staging.path().join(rel_path);
            let dst = output_dir.join(rel_path);

            // backup existing file before overwriting
            if dst.is_file() {
                let backup_path = backup_dir.join(rel_path);
                if let Some(p) = backup_path.parent() {
                    fs_err::create_dir_all(p)
                        .with_context(|| format!("failed to create backup dir {}", p.display()))?;
                }
                fs_err::copy(&dst, &backup_path)
                    .with_context(|| format!("failed to backup {}", dst.display()))?;
                backed_up.push((backup_path, dst.clone()));
            }

            if let Some(p) = dst.parent() {
                fs_err::create_dir_all(p)
                    .with_context(|| format!("failed to create {}", p.display()))?;
            }
            if fs_err::rename(&src, &dst).is_err() {
                fs_err::copy(&src, &dst).with_context(|| {
                    format!("failed to copy {} -> {}", src.display(), dst.display())
                })?;
            }
            written.push(dst);
        }
        Ok(())
    })();

    if let Err(err) = commit_result {
        // restore backed-up files to their original locations
        for (backup_path, original_path) in &backed_up {
            if let Err(e) = fs_err::copy(backup_path, original_path) {
                tracing::warn!(
                    backup = %backup_path.display(),
                    original = %original_path.display(),
                    error = %e,
                    "rollback: failed to restore file from backup"
                );
            }
        }
        // remove files that were newly created (had no backup)
        let backed_up_originals: std::collections::HashSet<&PathBuf> =
            backed_up.iter().map(|(_, orig)| orig).collect();
        for path in &written {
            if !backed_up_originals.contains(path) {
                if let Err(e) = fs_err::remove_file(path) {
                    tracing::debug!(path = %path.display(), error = %e, "rollback: failed to remove new file");
                }
            }
        }
        return Err(err.context("cast commit failed, rolled back written files"));
    }

    Ok(written)
}
