//! `theta check` — validate `theta.toml` and materialized dependencies.

#![allow(clippy::print_stdout)]

use std::path::Path;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use theta_cast::caster_for;
use theta_cli::{CheckArgs, OutputFormat};
use theta_harness::HarnessTarget;
use theta_manifest::{collect_document_diagnostics, read_document, read_manifest, schema_version};
use theta_schema::{DiagLevel, Diagnostic, ThetaManifest, Validate, ValidateContent};

use crate::resolve::{check_refs, resolve_content};

pub(crate) fn execute(args: CheckArgs, manifest_path: &Path) -> Result<()> {
    super::require_manifest(manifest_path)?;
    let manifest_label = manifest_label(manifest_path);
    let strict_materialization = !args.skip_materialization;
    let json = matches!(args.output_format, OutputFormat::Json);

    let doc = read_document(manifest_path)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    schema_version(&doc)?;

    let mut diags = Vec::new();
    collect_document_diagnostics(&doc, &mut diags);

    if args.schema_only {
        match read_manifest(manifest_path) {
            Ok(manifest) => manifest.validate(&mut diags),
            Err(e) => {
                diags.push(Diagnostic::error("[manifest]", format!("{e}")));
            }
        }
        if json {
            return finish_check_json(&diags);
        }
        return report_schema_only(&diags, &manifest_label);
    }

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to validate {}", manifest_path.display()))?;

    let project_dir = super::project_dir(manifest_path)?;

    manifest.validate(&mut diags);

    check_refs(&manifest, project_dir, strict_materialization, &mut diags);
    let resolved = resolve_content(&manifest, project_dir);
    manifest.validate_content("", &resolved, &mut diags);

    check_lock_and_materialization(manifest_path, &manifest, project_dir, &mut diags);

    for target in HarnessTarget::all() {
        let caster = caster_for(target);
        diags.extend(caster.validate_config(&manifest));
    }

    if json {
        return finish_check_json(&diags);
    }
    finish_check(&diags, &manifest_label)
}

fn finish_check_json(diags: &[Diagnostic]) -> Result<()> {
    let errors = diags.iter().filter(|d| d.level == DiagLevel::Error).count();
    let warnings = diags.iter().filter(|d| d.level == DiagLevel::Warn).count();
    let hints = diags.iter().filter(|d| d.level == DiagLevel::Hint).count();
    let output = serde_json::json!({
        "valid": errors == 0,
        "errors": errors,
        "warnings": warnings,
        "hints": hints,
        "diagnostics": diags,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    if errors > 0 {
        anyhow::bail!("validation failed with {errors} error(s)");
    }
    Ok(())
}

fn check_lock_and_materialization(
    manifest_path: &Path,
    manifest: &ThetaManifest,
    project_dir: &Path,
    diags: &mut Vec<Diagnostic>,
) {
    let manifest_bytes = match fs_err::read(manifest_path) {
        Ok(b) => b,
        Err(e) => {
            tracing::warn!(path = %manifest_path.display(), error = %e, "failed to read manifest for lock/materialization check");
            return;
        }
    };
    diags.extend(theta_install::check_consistency(
        &manifest_bytes,
        manifest,
        project_dir,
    ));
}

fn report_schema_only(diags: &[Diagnostic], manifest_label: &str) -> Result<()> {
    let (errors, warnings) = super::report_diagnostics(diags);
    if errors > 0 {
        anyhow::bail!("{manifest_label} has {errors} error(s) and {warnings} warning(s)");
    }

    if warnings > 0 {
        anstream::eprintln!(
            "{} {} passed schema checks with {} warning(s)",
            "ok".green().bold(),
            manifest_label.cyan(),
            warnings,
        );
    } else {
        anstream::eprintln!(
            "{} {} passed schema checks",
            "ok".green().bold(),
            manifest_label.cyan(),
        );
    }
    Ok(())
}

fn finish_check(diags: &[Diagnostic], manifest_label: &str) -> Result<()> {
    let (errors, warnings) = super::report_diagnostics(diags);

    if errors > 0 {
        anyhow::bail!("{manifest_label} has {errors} error(s) and {warnings} warning(s)");
    }

    if warnings > 0 {
        anstream::eprintln!(
            "{} {} is valid with {} warning(s)",
            "ok".green().bold(),
            manifest_label.cyan(),
            warnings,
        );
    } else {
        anstream::eprintln!("{} {} is valid", "ok".green().bold(), manifest_label.cyan(),);
    }

    Ok(())
}

fn manifest_label(manifest_path: &Path) -> String {
    manifest_path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| manifest_path.display().to_string(), ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use theta_lock::{build_lock, write_lock};
    use theta_manifest::read_manifest;
    use theta_schema::{
        ApplyMode, DiagLevel, Instructions, LocalOrGitRef, LocalPathRef, Rule, minimal_manifest,
    };

    // set up a project with system prompt + one rule, locked + synced
    fn setup_locked_synced_project() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().to_path_buf();

        let mut manifest = minimal_manifest("test-agent");
        manifest.agent.description = "Integration test agent".to_string();
        manifest.agent.model = Some("claude-sonnet-4-20250514".to_string());
        let mut rules = BTreeMap::new();
        rules.insert(
            "safety".to_string(),
            Rule {
                src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/safety.md")),
                summary: None,
                description: Some("Safety rule".to_string()),
                apply: ApplyMode::Always,
                apply_to: None,
            },
        );
        manifest.instructions = Some(Instructions {
            system: Some("instructions/system.md".into()),
            rules: Some(rules),
        });

        let manifest_toml = toml::to_string_pretty(&manifest).unwrap();
        let manifest_path = project.join("theta.toml");
        fs_err::write(&manifest_path, &manifest_toml).unwrap();

        // create source files
        fs_err::create_dir_all(project.join("instructions/rules")).unwrap();
        fs_err::write(
            project.join("instructions/rules/safety.md"),
            "Never produce harmful output.",
        )
        .unwrap();
        fs_err::write(
            project.join("instructions/system.md"),
            "You are a helpful AI assistant.",
        )
        .unwrap();

        // lock
        let manifest_bytes = fs_err::read(&manifest_path).unwrap();
        let parsed = read_manifest(&manifest_path).unwrap();
        let git_cache = project.join(".git-cache");
        fs_err::create_dir_all(&git_cache).unwrap();
        let lock = build_lock(&parsed, &manifest_bytes, &project, &git_cache).unwrap();
        let lock_path = project.join(theta_static::LOCKFILE);
        write_lock(&lock_path, &lock).unwrap();

        // materialize into .theta/
        let theta_dir = project.join(theta_static::DOT_THETA_DIR);
        fs_err::create_dir_all(theta_dir.join("rules")).unwrap();
        fs_err::copy(
            project.join("instructions/system.md"),
            theta_dir.join(theta_static::SYSTEM_FILE_NAME),
        )
        .unwrap();
        fs_err::copy(
            project.join("instructions/rules/safety.md"),
            theta_dir.join("rules/safety.md"),
        )
        .unwrap();

        (dir, project)
    }

    #[test]
    fn clean_project_produces_no_lock_diagnostics() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");
        let manifest = read_manifest(&manifest_path).unwrap();
        let mut diags = Vec::new();

        check_lock_and_materialization(&manifest_path, &manifest, &project, &mut diags);

        let lock_diags: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.path.starts_with("theta.lock")
                    || d.path.starts_with(".theta/")
                    || d.path.starts_with(".theta\\")
            })
            .collect();
        assert!(
            lock_diags.is_empty(),
            "expected no diagnostics, got: {lock_diags:?}"
        );
    }

    #[test]
    fn stale_lock_warns_and_skips_drift() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");

        // remove a rule from theta.toml without re-locking - makes lock stale
        // AND creates an orphaned lock entry that would trigger drift if checked
        let mut manifest = read_manifest(&manifest_path).unwrap();
        manifest
            .instructions
            .as_mut()
            .unwrap()
            .rules
            .as_mut()
            .unwrap()
            .remove("safety");
        manifest.agent.description = "Changed description".to_string();
        let new_toml = toml::to_string_pretty(&manifest).unwrap();
        fs_err::write(&manifest_path, &new_toml).unwrap();

        let manifest = read_manifest(&manifest_path).unwrap();
        let mut diags = Vec::new();
        check_lock_and_materialization(&manifest_path, &manifest, &project, &mut diags);

        // stale warning MUST be present
        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == "theta.lock"
                && d.message.contains("stale")),
            "expected stale lock warning, got: {diags:?}"
        );

        // drift diagnostics MUST NOT be present (misleading on stale lock)
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("orphaned lock entry")),
            "stale lock should skip drift checks, but got drift diagnostic: {diags:?}"
        );
    }

    #[test]
    fn missing_lock_hints() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");

        fs_err::remove_file(project.join(theta_static::LOCKFILE)).unwrap();

        let manifest = read_manifest(&manifest_path).unwrap();
        let mut diags = Vec::new();
        check_lock_and_materialization(&manifest_path, &manifest, &project, &mut diags);

        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Hint && d.path == "theta.lock"),
            "expected missing lock hint, got: {diags:?}"
        );
    }

    #[test]
    fn missing_theta_dir_hints() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");

        fs_err::remove_dir_all(project.join(theta_static::DOT_THETA_DIR)).unwrap();

        let manifest = read_manifest(&manifest_path).unwrap();
        let mut diags = Vec::new();
        check_lock_and_materialization(&manifest_path, &manifest, &project, &mut diags);

        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Hint && d.path == ".theta/"),
            "expected missing .theta/ hint, got: {diags:?}"
        );
    }

    #[test]
    fn missing_materialized_file_warns() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");

        let theta_dir = project.join(theta_static::DOT_THETA_DIR);
        fs_err::remove_file(theta_dir.join("rules/safety.md")).unwrap();

        let manifest = read_manifest(&manifest_path).unwrap();
        let mut diags = Vec::new();
        check_lock_and_materialization(&manifest_path, &manifest, &project, &mut diags);

        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == ".theta/rules/safety.md"
                && d.message.contains("missing")),
            "expected missing materialized file warning, got: {diags:?}"
        );
    }

    #[test]
    fn orphaned_lock_entry_warns_on_fresh_lock() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");

        // remove a rule from theta.toml AND re-lock so the lock is fresh
        // but the old lock still has the entry
        let mut manifest = read_manifest(&manifest_path).unwrap();
        manifest
            .instructions
            .as_mut()
            .unwrap()
            .rules
            .as_mut()
            .unwrap()
            .remove("safety");
        let new_toml = toml::to_string_pretty(&manifest).unwrap();
        fs_err::write(&manifest_path, &new_toml).unwrap();

        // DON'T re-lock - but we need the lock to be "fresh" for drift to run
        // Force-write a lock that has the old rule but with the NEW manifest hash
        let manifest_bytes = fs_err::read(&manifest_path).unwrap();
        let lock_path = project.join(theta_static::LOCKFILE);
        let mut lock = theta_lock::read_lock(&lock_path).unwrap();
        lock.meta.manifest_hash = theta_lock::manifest_hash(&manifest_bytes).unwrap();
        write_lock(&lock_path, &lock).unwrap();

        let manifest = read_manifest(&manifest_path).unwrap();
        let mut diags = Vec::new();
        check_lock_and_materialization(&manifest_path, &manifest, &project, &mut diags);

        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == "theta.lock"
                && d.message.contains("orphaned")
                && d.message.contains("safety")),
            "expected orphaned lock entry warning, got: {diags:?}"
        );
    }
}
