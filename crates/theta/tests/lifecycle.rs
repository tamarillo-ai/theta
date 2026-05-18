//! Lock --> sync --> cast lifecycle integration tests.

use std::collections::BTreeMap;
use std::path::Path;

use theta_cast::{Caster, write_cast_output};
use theta_lock::{build_lock, is_stale, read_lock, write_lock};
use theta_manifest::read_manifest;
use theta_schema::{ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, minimal_manifest};
use theta_static::LOCKFILE;

// dummy git cache dir for tests with only local sources
fn dummy_git_cache(project: &Path) -> std::path::PathBuf {
    let p = project.join(".git-cache");
    fs_err::create_dir_all(&p).ok();
    p
}

/// Helper: write a `theta.toml` into a tempdir and return `(manifest_path, project_dir)`.
fn setup_project_with_rule() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_path_buf();

    // create manifest
    let mut manifest = minimal_manifest("test-agent");
    manifest.agent.description = "Integration test agent".to_string();
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

    (dir, project)
}

#[test]
fn lock_creates_lockfile_with_correct_entries() {
    let (_dir, project) = setup_project_with_rule();
    let manifest_path = project.join("theta.toml");
    let manifest_bytes = fs_err::read(&manifest_path).unwrap();
    let manifest = read_manifest(&manifest_path).unwrap();

    let lock = build_lock(
        &manifest,
        &manifest_bytes,
        &project,
        &dummy_git_cache(&project),
    )
    .unwrap();

    // check meta
    assert!(lock.meta.manifest_hash.to_string().starts_with("sha256:"));

    // check instructions
    let instr = lock.instructions.as_ref().unwrap();
    assert!(instr.system.is_some());
    assert!(
        instr
            .system
            .as_ref()
            .unwrap()
            .content_hash
            .to_string()
            .starts_with("sha256:")
    );
    assert_eq!(instr.rules.len(), 1);
    assert!(instr.rules.contains_key("safety"));
}

#[test]
fn lock_then_sync_materializes_theta_dir() {
    let (_dir, project) = setup_project_with_rule();
    let manifest_path = project.join("theta.toml");
    let manifest_bytes = fs_err::read(&manifest_path).unwrap();
    let manifest = read_manifest(&manifest_path).unwrap();

    // lock
    let lock = build_lock(
        &manifest,
        &manifest_bytes,
        &project,
        &dummy_git_cache(&project),
    )
    .unwrap();
    let lock_path = project.join(LOCKFILE);
    write_lock(&lock_path, &lock).unwrap();

    // simulate sync: copy source files to .theta/ layout
    let theta_dir = project.join(".theta");
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

    // verify materialized files exist
    assert!(theta_dir.join(theta_static::SYSTEM_FILE_NAME).exists());
    assert!(theta_dir.join("rules/safety.md").exists());

    let system = fs_err::read_to_string(theta_dir.join(theta_static::SYSTEM_FILE_NAME)).unwrap();
    assert_eq!(system, "You are a helpful AI assistant.");
}

#[test]
fn lock_then_sync_then_cast_produces_correct_output() {
    let (_dir, project) = setup_project_with_rule();
    let manifest_path = project.join("theta.toml");
    let manifest_bytes = fs_err::read(&manifest_path).unwrap();
    let manifest = read_manifest(&manifest_path).unwrap();

    // lock
    let lock = build_lock(
        &manifest,
        &manifest_bytes,
        &project,
        &dummy_git_cache(&project),
    )
    .unwrap();
    let lock_path = project.join(LOCKFILE);
    write_lock(&lock_path, &lock).unwrap();

    // sync (manual materialization)
    let theta_dir = project.join(".theta");
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

    // cast from .theta/
    let output = project.join("output");
    let caster = theta_cast::ClaudeCode;
    let files = caster.cast_files(&manifest, &theta_dir).unwrap();
    let written = write_cast_output(&files, &output).unwrap();

    assert_eq!(written.len(), 2);

    let claude_md = fs_err::read_to_string(output.join("CLAUDE.md")).unwrap();
    assert!(claude_md.contains("You are a helpful AI assistant."));
    assert!(
        !claude_md.contains("# test-agent"),
        "CLAUDE.md must not contain a synthetic identity header"
    );

    let rule_file = fs_err::read_to_string(output.join(".claude/rules/safety.md")).unwrap();
    assert!(rule_file.contains("Never produce harmful output."));
}

#[test]
fn staleness_detected_after_manifest_change() {
    let (_dir, project) = setup_project_with_rule();
    let manifest_path = project.join("theta.toml");
    let manifest_bytes = fs_err::read(&manifest_path).unwrap();
    let manifest = read_manifest(&manifest_path).unwrap();

    let lock = build_lock(
        &manifest,
        &manifest_bytes,
        &project,
        &dummy_git_cache(&project),
    )
    .unwrap();
    let lock_path = project.join(LOCKFILE);
    write_lock(&lock_path, &lock).unwrap();

    // not stale initially
    assert!(
        is_stale(&lock, &manifest_bytes, &project)
            .unwrap()
            .is_none()
    );

    // modify manifest
    let mut new_manifest = manifest.clone();
    new_manifest.agent.description = "Changed description".to_string();
    let new_toml = toml::to_string_pretty(&new_manifest).unwrap();
    fs_err::write(&manifest_path, &new_toml).unwrap();
    let new_bytes = fs_err::read(&manifest_path).unwrap();

    // now stale
    assert!(is_stale(&lock, &new_bytes, &project).unwrap().is_some());
}

#[test]
fn lock_round_trips_through_file() {
    let (_dir, project) = setup_project_with_rule();
    let manifest_path = project.join("theta.toml");
    let manifest_bytes = fs_err::read(&manifest_path).unwrap();
    let manifest = read_manifest(&manifest_path).unwrap();

    let lock = build_lock(
        &manifest,
        &manifest_bytes,
        &project,
        &dummy_git_cache(&project),
    )
    .unwrap();
    let lock_path = project.join(LOCKFILE);
    write_lock(&lock_path, &lock).unwrap();

    let loaded = read_lock(&lock_path).unwrap();
    assert_eq!(lock, loaded);
}
