//! Lock satisfaction and materialization checks.
//!
//! Heavily inspired by uv's pattern: structured result types that the command layer
//! converts to user-facing diagnostics.

use std::collections::BTreeSet;
use std::path::Path;

use theta_lock::LockFile;
use theta_schema::Diagnostic;
use theta_static::ThetaProjectLayout as Layout;

pub fn check_consistency(
    manifest_bytes: &[u8],
    manifest: &theta_schema::ThetaManifest,
    project_dir: &Path,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let lock_path = project_dir.join(theta_static::LOCKFILE);
    let theta_dir = project_dir.join(theta_static::DOT_THETA_DIR);

    if !lock_path.exists() {
        diags.push(Diagnostic::hint(
            theta_static::LOCKFILE,
            "no lockfile found - run `theta sync` to lock and materialize dependencies",
        ));
        if !theta_dir.exists() {
            diags.push(Diagnostic::hint(
                ".theta/",
                "not materialized - run `theta sync` to populate",
            ));
        }
        return diags;
    }

    let lock = match theta_lock::read_lock(&lock_path) {
        Ok(l) => l,
        Err(e) => {
            diags.push(Diagnostic::warn(
                theta_static::LOCKFILE,
                format!("failed to read lockfile: {e}"),
            ));
            return diags;
        }
    };

    let stale_reason = match theta_lock::is_stale(&lock, manifest_bytes, project_dir) {
        Ok(reason) => reason,
        Err(e) => {
            diags.push(Diagnostic::error(
                theta_static::MANIFEST_FILE_NAME,
                format!("cannot compute manifest hash: {e}"),
            ));
            return diags;
        }
    };

    if let Some(ref reason) = stale_reason {
        diags.push(Diagnostic::warn(
            theta_static::LOCKFILE,
            format!("lock is stale: {reason}. Run `theta sync` to update."),
        ));
    }

    if stale_reason.is_none() {
        check_lock_manifest_drift(&lock, manifest, &mut diags);
        check_source_content_drift(&lock, project_dir, &mut diags);
    }

    if !theta_dir.exists() {
        diags.push(Diagnostic::hint(
            ".theta/",
            "not materialized - run `theta sync` to populate",
        ));
        return diags;
    }

    check_materialization(&lock, &theta_dir, &mut diags);

    diags
}

fn check_lock_manifest_drift(
    lock: &LockFile,
    manifest: &theta_schema::ThetaManifest,
    diags: &mut Vec<Diagnostic>,
) {
    if let Some(ref lock_instr) = lock.instructions {
        if lock_instr.system.is_some() {
            let manifest_has_system = manifest
                .instructions
                .as_ref()
                .is_some_and(|i| i.system.is_some());
            if !manifest_has_system {
                diags.push(Diagnostic::warn(
                    theta_static::LOCKFILE,
                    "orphaned lock entry: instructions.system is locked but not declared in theta.toml",
                ));
            }
        }

        let manifest_rules: BTreeSet<&str> = manifest
            .instructions
            .as_ref()
            .and_then(|i| i.rules.as_ref())
            .map(|r| r.keys().map(String::as_str).collect())
            .unwrap_or_default();

        check_orphaned_locked(
            "instructions.rules",
            &lock_instr.rules,
            &manifest_rules,
            diags,
        );
    }

    let manifest_skills: BTreeSet<&str> = manifest
        .skills
        .as_ref()
        .map(|s| s.keys().map(String::as_str).collect())
        .unwrap_or_default();

    check_orphaned_locked("skills", &lock.skills, &manifest_skills, diags);

    let manifest_subagent_names: BTreeSet<&str> = manifest
        .subagents
        .as_ref()
        .map(|subs| subs.iter().map(|s| s.name.as_str()).collect())
        .unwrap_or_default();

    check_orphaned_locked(
        "subagents",
        &lock.subagents,
        &manifest_subagent_names,
        diags,
    );
}

fn check_orphaned_locked<V>(
    kind: &str,
    lock_map: &std::collections::BTreeMap<String, V>,
    manifest_keys: &BTreeSet<&str>,
    diags: &mut Vec<Diagnostic>,
) {
    for name in lock_map.keys() {
        if !manifest_keys.contains(name.as_str()) {
            diags.push(Diagnostic::warn(
                theta_static::LOCKFILE,
                format!(
                    "orphaned lock entry: {kind}.{name} is locked but not declared in theta.toml"
                ),
            ));
        }
    }
}

/// Local filesystem path for a locked source, or `None` for non-local sources.
///
/// git, system, registry, and URL sources return `None` — drift checking
/// **MUST NOT** apply to these because their content is immutable once locked:
///
/// - git sources are pinned to an exact commit SHA in `theta.lock`.
///   Content at a given commit never changes. Re-resolution requires
///   explicit `theta lock`.
///   uv: [`Source::is_immutable`](https://github.com/astral-sh/uv/blob/main/crates/uv-resolver/src/lock/mod.rs#L3993-L4002)
///   cargo: [`Revision::Locked`](https://github.com/rust-lang/cargo/blob/master/src/cargo/sources/git/source.rs#L70-L89)
/// - system/registry sources are managed externally and opaque to the project
fn local_source_path(source: &theta_lock::LockedSource) -> Option<&str> {
    match source {
        theta_lock::LockedSource::Path { path } => Some(path.as_str()),
        _ => None,
    }
}

fn check_source_content_drift(lock: &LockFile, project_dir: &Path, diags: &mut Vec<Diagnostic>) {
    if let Some(ref instr) = lock.instructions {
        if let Some((path, sys)) = instr
            .system
            .as_ref()
            .and_then(|s| local_source_path(&s.source).map(|p| (p, s)))
        {
            if let Ok(data) = fs_err::read(project_dir.join(path)) {
                if theta_lock::content_hash(&data) != sys.content_hash {
                    diags.push(Diagnostic::warn(
                        path,
                        "source file changed since last lock. Run `theta sync` to update.",
                    ));
                }
            }
        }
        for (name, entry) in &instr.rules {
            let Some(path) = local_source_path(&entry.source) else {
                continue;
            };
            let Ok(data) = fs_err::read(project_dir.join(path)) else {
                continue;
            };
            if theta_lock::content_hash(&data) != entry.content_hash {
                diags.push(Diagnostic::warn(
                    path,
                    format!(
                        "rule '{name}' source changed since last lock. Run `theta sync` to update."
                    ),
                ));
            }
        }
    }
    for (name, entry) in &lock.skills {
        let Some(path) = local_source_path(&entry.source) else {
            continue;
        };
        let full = project_dir.join(path);
        if !full.is_dir() {
            continue;
        }
        let Ok(hash) = theta_lock::skill_content_hash(&full) else {
            continue;
        };
        if hash != entry.content_hash {
            diags.push(Diagnostic::warn(
                path,
                format!(
                    "skill '{name}' source changed since last lock. run `theta sync` to update."
                ),
            ));
        }
    }
}

fn check_materialization(lock: &LockFile, theta_dir: &Path, diags: &mut Vec<Diagnostic>) {
    let expected = check_completeness(lock, theta_dir, diags);
    check_orphans(&expected, theta_dir, diags);
}

fn check_completeness(
    lock: &LockFile,
    theta_dir: &Path,
    diags: &mut Vec<Diagnostic>,
) -> BTreeSet<String> {
    let mut expected = BTreeSet::new();

    if let Some(ref instr) = lock.instructions {
        if let Some(ref sys) = instr.system {
            let rel = theta_static::SYSTEM_FILE_NAME.to_string();
            check_materialized_file(theta_dir, &rel, &sys.content_hash, diags);
            expected.insert(rel);
        }

        for (name, entry) in &instr.rules {
            let rel = Layout::rule_rel(name);
            check_materialized_file(theta_dir, &rel, &entry.content_hash, diags);
            expected.insert(rel);
        }
    }

    for (name, entry) in &lock.skills {
        let rel = Layout::skill_rel(name);
        check_materialized_skill_dir(theta_dir, &rel, &entry.content_hash, diags);
        expected.insert(rel);
    }

    for (name, entry) in &lock.subagents {
        match entry {
            theta_lock::SubagentLock::Ref { resource, .. } => {
                let rel = format!(
                    "{}/{}",
                    Layout::subagent_rel(name),
                    theta_static::MANIFEST_FILE_NAME,
                );
                check_materialized_file(theta_dir, &rel, &resource.content_hash, diags);
                expected.insert(Layout::subagent_rel(name));
            }
            theta_lock::SubagentLock::Inline { prompt } => {
                let rel = theta_static::subagent_system_rel(name);
                check_materialized_file(theta_dir, &rel, &prompt.content_hash, diags);
                expected.insert(Layout::subagent_rel(name));
            }
            other => {
                diags.push(Diagnostic::warn(
                    format!("subagents.{name}"),
                    format!("unknown lock variant {other:?}, skipped check"),
                ));
            }
        }
    }

    expected
}

fn check_materialized_file(
    theta_dir: &Path,
    rel: &str,
    expected_hash: &theta_lock::ContentHash,
    diags: &mut Vec<Diagnostic>,
) {
    let full = theta_dir.join(rel);
    if !full.exists() {
        diags.push(Diagnostic::warn(
            format!(".theta/{rel}"),
            "missing materialized file - run `theta sync` to populate",
        ));
        return;
    }
    match fs_err::read(&full) {
        Ok(data) => {
            let actual = theta_lock::content_hash(&data);
            if actual != *expected_hash {
                diags.push(Diagnostic::warn(
                    format!(".theta/{rel}"),
                    "content hash mismatch - materialized file differs from lock. Run `theta sync` to update.",
                ));
            }
        }
        Err(e) => {
            diags.push(Diagnostic::warn(
                format!(".theta/{rel}"),
                format!("failed to read materialized file: {e}"),
            ));
        }
    }
}

fn check_materialized_skill_dir(
    theta_dir: &Path,
    rel: &str,
    expected_hash: &theta_lock::ContentHash,
    diags: &mut Vec<Diagnostic>,
) {
    let full = theta_dir.join(rel);
    if !full.exists() {
        diags.push(Diagnostic::warn(
            format!(".theta/{rel}"),
            "missing materialized skill directory - run `theta sync` to populate",
        ));
        return;
    }
    match theta_lock::skill_content_hash(&full) {
        Ok(actual) => {
            if actual != *expected_hash {
                diags.push(Diagnostic::warn(
                    format!(".theta/{rel}"),
                    "content hash mismatch - materialized skill differs from lock. Run `theta sync` to update.",
                ));
            }
        }
        Err(e) => {
            diags.push(Diagnostic::warn(
                format!(".theta/{rel}"),
                format!("failed to hash materialized skill directory: {e}"),
            ));
        }
    }
}

fn check_orphans(expected: &BTreeSet<String>, theta_dir: &Path, diags: &mut Vec<Diagnostic>) {
    if theta_dir.join(theta_static::SYSTEM_FILE_NAME).exists()
        && !expected.contains(theta_static::SYSTEM_FILE_NAME)
    {
        diags.push(Diagnostic::warn(
            format!(".theta/{}", theta_static::SYSTEM_FILE_NAME),
            "orphaned system prompt - no corresponding lock entry. Run `theta sync` to clean up.",
        ));
    }

    scan_orphaned_entries(
        &theta_dir.join(theta_static::RULES_DIR),
        theta_static::RULES_DIR,
        "rule",
        expected,
        false,
        diags,
    );

    scan_orphaned_entries(
        &theta_dir.join(theta_static::SKILLS_DIR),
        theta_static::SKILLS_DIR,
        "skill",
        expected,
        true,
        diags,
    );

    scan_orphaned_entries(
        &theta_dir.join(theta_static::SUBAGENTS_DIR_NAME),
        "subagents",
        "subagent",
        expected,
        true,
        diags,
    );
}

fn scan_orphaned_entries(
    dir: &Path,
    prefix: &str,
    kind: &str,
    expected: &BTreeSet<String>,
    is_dir_entry: bool,
    diags: &mut Vec<Diagnostic>,
) {
    let Ok(entries) = fs_err::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            diags.push(Diagnostic::warn(
                format!(".theta/{prefix}/{}", name.to_string_lossy()),
                "non-UTF-8 filename in .theta/ - skipped during orphan check",
            ));
            continue;
        };
        let rel = format!("{prefix}/{name_str}");

        if is_dir_entry {
            if entry.path().is_dir() && !expected.contains(&rel) {
                diags.push(Diagnostic::warn(
                    format!(".theta/{rel}"),
                    format!(
                        "orphaned materialized {kind} - no corresponding lock entry. Run `theta sync` to clean up.",
                    ),
                ));
            }
        } else if entry.path().is_file() && !expected.contains(&rel) {
            diags.push(Diagnostic::warn(
                format!(".theta/{rel}"),
                format!(
                    "orphaned materialized {kind} - no corresponding lock entry. Run `theta sync` to clean up.",
                ),
            ));
        }
    }
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

        let manifest_bytes = fs_err::read(&manifest_path).unwrap();
        let parsed = read_manifest(&manifest_path).unwrap();
        let git_cache = project.join(".git-cache");
        fs_err::create_dir_all(&git_cache).unwrap();
        let lock = build_lock(&parsed, &manifest_bytes, &project, &git_cache).unwrap();
        let lock_path = project.join(theta_static::LOCKFILE);
        write_lock(&lock_path, &lock).unwrap();

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

    fn manifest_bytes_and_parsed(
        project: &std::path::Path,
    ) -> (Vec<u8>, theta_schema::ThetaManifest) {
        let manifest_path = project.join("theta.toml");
        let bytes = fs_err::read(&manifest_path).unwrap();
        let manifest = read_manifest(&manifest_path).unwrap();
        (bytes, manifest)
    }

    #[test]
    fn clean_project_produces_no_diagnostics() {
        let (_dir, project) = setup_locked_synced_project();
        let (bytes, manifest) = manifest_bytes_and_parsed(&project);

        let diags = check_consistency(&bytes, &manifest, &project);
        assert!(diags.is_empty(), "expected no diagnostics, got: {diags:?}");
    }

    #[test]
    fn missing_lock_hints() {
        let (_dir, project) = setup_locked_synced_project();
        fs_err::remove_file(project.join(theta_static::LOCKFILE)).unwrap();
        let (bytes, manifest) = manifest_bytes_and_parsed(&project);

        let diags = check_consistency(&bytes, &manifest, &project);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Hint && d.path == "theta.lock"),
            "expected missing lock hint, got: {diags:?}"
        );
    }

    #[test]
    fn missing_lock_and_theta_dir_hints() {
        let (_dir, project) = setup_locked_synced_project();
        fs_err::remove_file(project.join(theta_static::LOCKFILE)).unwrap();
        fs_err::remove_dir_all(project.join(theta_static::DOT_THETA_DIR)).unwrap();
        let (bytes, manifest) = manifest_bytes_and_parsed(&project);

        let diags = check_consistency(&bytes, &manifest, &project);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Hint && d.path == "theta.lock"),
            "expected missing lock hint, got: {diags:?}"
        );
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Hint && d.path == ".theta/"),
            "expected missing .theta/ hint, got: {diags:?}"
        );
    }

    #[test]
    fn stale_lock_warns_and_skips_drift() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");

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

        let (bytes, manifest) = manifest_bytes_and_parsed(&project);
        let diags = check_consistency(&bytes, &manifest, &project);

        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == "theta.lock"
                && d.message.contains("stale")),
            "expected stale warning, got: {diags:?}"
        );
        assert!(
            !diags
                .iter()
                .any(|d| d.message.contains("orphaned lock entry")),
            "stale lock should skip drift, got: {diags:?}"
        );
    }

    #[test]
    fn drift_orphaned_lock_rule_warns() {
        let (_dir, project) = setup_locked_synced_project();
        let manifest_path = project.join("theta.toml");

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

        let manifest_bytes = fs_err::read(&manifest_path).unwrap();
        let lock_path = project.join(theta_static::LOCKFILE);
        let mut lock = theta_lock::read_lock(&lock_path).unwrap();
        lock.meta.manifest_hash = theta_lock::manifest_hash(&manifest_bytes).unwrap();
        write_lock(&lock_path, &lock).unwrap();

        let (bytes, manifest) = manifest_bytes_and_parsed(&project);
        let diags = check_consistency(&bytes, &manifest, &project);

        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.message.contains("orphaned")
                && d.message.contains("safety")),
            "expected orphaned lock entry warning, got: {diags:?}"
        );
    }

    #[test]
    fn missing_theta_dir_hints() {
        let (_dir, project) = setup_locked_synced_project();
        fs_err::remove_dir_all(project.join(theta_static::DOT_THETA_DIR)).unwrap();
        let (bytes, manifest) = manifest_bytes_and_parsed(&project);

        let diags = check_consistency(&bytes, &manifest, &project);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Hint && d.path == ".theta/"),
            "expected missing .theta/ hint, got: {diags:?}"
        );
    }

    #[test]
    fn materialization_missing_file_warns() {
        let (_dir, project) = setup_locked_synced_project();
        let theta_dir = project.join(theta_static::DOT_THETA_DIR);
        fs_err::remove_file(theta_dir.join("rules/safety.md")).unwrap();
        let (bytes, manifest) = manifest_bytes_and_parsed(&project);

        let diags = check_consistency(&bytes, &manifest, &project);
        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == ".theta/rules/safety.md"
                && d.message.contains("missing")),
            "expected missing file warning, got: {diags:?}"
        );
    }

    #[test]
    fn materialization_hash_mismatch_warns() {
        let (_dir, project) = setup_locked_synced_project();
        let theta_dir = project.join(theta_static::DOT_THETA_DIR);
        fs_err::write(
            theta_dir.join("rules/safety.md"),
            "MODIFIED - hash will differ",
        )
        .unwrap();
        let (bytes, manifest) = manifest_bytes_and_parsed(&project);

        let diags = check_consistency(&bytes, &manifest, &project);
        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == ".theta/rules/safety.md"
                && d.message.contains("hash mismatch")),
            "expected hash mismatch warning, got: {diags:?}"
        );
    }

    #[test]
    fn materialization_orphaned_rule_warns() {
        let (_dir, project) = setup_locked_synced_project();
        let theta_dir = project.join(theta_static::DOT_THETA_DIR);
        fs_err::write(theta_dir.join("rules/stale-rule.md"), "orphan").unwrap();
        let (bytes, manifest) = manifest_bytes_and_parsed(&project);

        let diags = check_consistency(&bytes, &manifest, &project);
        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == ".theta/rules/stale-rule.md"
                && d.message.contains("orphaned materialized rule")),
            "expected orphaned rule warning, got: {diags:?}"
        );
    }

    #[test]
    fn stale_from_rule_change_includes_reason() {
        let (_dir, project) = setup_locked_synced_project();

        // edit a rule file without re-locking
        fs_err::write(
            project.join("instructions/rules/safety.md"),
            "Completely rewritten rule content.",
        )
        .unwrap();

        let (bytes, manifest) = manifest_bytes_and_parsed(&project);
        let diags = check_consistency(&bytes, &manifest, &project);

        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == "theta.lock"
                && d.message.contains("rule")
                && d.message.contains("safety")),
            "expected stale warning mentioning rule 'safety', got: {diags:?}"
        );
    }

    #[test]
    fn stale_from_system_instruction_change_includes_reason() {
        let (_dir, project) = setup_locked_synced_project();

        // edit system instruction without re-locking
        fs_err::write(
            project.join("instructions/system.md"),
            "Completely different system prompt.",
        )
        .unwrap();

        let (bytes, manifest) = manifest_bytes_and_parsed(&project);
        let diags = check_consistency(&bytes, &manifest, &project);

        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == "theta.lock"
                && d.message.contains("system instruction")),
            "expected stale warning mentioning system instruction, got: {diags:?}"
        );
    }

    #[test]
    fn stale_from_manifest_change_includes_reason() {
        let (_dir, project) = setup_locked_synced_project();

        // change the manifest without re-locking
        let manifest_path = project.join("theta.toml");
        let mut manifest = read_manifest(&manifest_path).unwrap();
        manifest.agent.description = "Totally new description".to_string();
        let new_toml = toml::to_string_pretty(&manifest).unwrap();
        fs_err::write(&manifest_path, &new_toml).unwrap();

        let (bytes, manifest) = manifest_bytes_and_parsed(&project);
        let diags = check_consistency(&bytes, &manifest, &project);

        assert!(
            diags.iter().any(|d| d.level == DiagLevel::Warn
                && d.path == "theta.lock"
                && d.message.contains("theta.toml changed")),
            "expected stale warning mentioning manifest change, got: {diags:?}"
        );
    }
}
