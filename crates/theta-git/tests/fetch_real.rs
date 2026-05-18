//! Integration tests for theta-git.
//!
//! Uses local bare repos as "remotes" for skill tests -- no network, fast,
//! deterministic. Also includes network tests against real GitHub repos,
//! skipped by default. Run with:
//!
//! ```bash
//! cargo nextest run -p theta-git --test fetch_real --features online-tests
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use theta_git::{GitFetcher, GitRef};

fn git(args: &[&str], cwd: &Path) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@test.com")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@test.com")
        .output()
        .expect("git should be on PATH");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// create a local bare repo with a SKILL.md at the root (repo == skill)
fn make_skill_repo(root: &Path) -> (PathBuf, String) {
    let work = root.join("work");
    fs_err::create_dir_all(&work).unwrap();
    git(&["init"], &work);
    git(&["checkout", "-b", "main"], &work);

    let skill_content = "\
---
name: test-skill
description: a test skill for integration tests
---

# test-skill

this is a test skill used by theta-git integration tests.
";
    fs_err::write(work.join("SKILL.md"), skill_content).unwrap();
    fs_err::create_dir_all(work.join("scripts")).unwrap();
    fs_err::write(work.join("scripts/helper.sh"), "#!/bin/sh\necho ok").unwrap();
    git(&["add", "."], &work);
    git(&["commit", "-m", "initial skill"], &work);
    let sha = git(&["rev-parse", "HEAD"], &work);

    let bare = root.join("remote.git");
    git(
        &[
            "clone",
            "--bare",
            &work.to_string_lossy(),
            &bare.to_string_lossy(),
        ],
        root,
    );
    (bare, sha)
}

// create a local bare repo with skills in subdirectories
fn make_multiskill_repo(root: &Path) -> (PathBuf, String) {
    let work = root.join("work");
    fs_err::create_dir_all(&work).unwrap();
    git(&["init"], &work);
    git(&["checkout", "-b", "main"], &work);

    fs_err::write(work.join("README.md"), "# multi-skill repo").unwrap();

    // skills/frontend-design
    let frontend = work.join("skills/frontend-design");
    fs_err::create_dir_all(&frontend).unwrap();
    fs_err::write(
        frontend.join("SKILL.md"),
        "---\nname: frontend-design\ndescription: design frontend\n---\n# frontend-design\n",
    )
    .unwrap();
    fs_err::write(frontend.join("reference.md"), "some reference material").unwrap();

    // skills/backend-api
    let backend = work.join("skills/backend-api");
    fs_err::create_dir_all(&backend).unwrap();
    fs_err::write(
        backend.join("SKILL.md"),
        "---\nname: backend-api\ndescription: api skill\n---\n# backend-api\n",
    )
    .unwrap();

    git(&["add", "."], &work);
    git(&["commit", "-m", "multi-skill repo"], &work);
    let sha = git(&["rev-parse", "HEAD"], &work);

    let bare = root.join("remote.git");
    git(
        &[
            "clone",
            "--bare",
            &work.to_string_lossy(),
            &bare.to_string_lossy(),
        ],
        root,
    );
    (bare, sha)
}

#[test]
fn fetch_local_repo() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, expected_sha) = make_skill_repo(dir.path());
    let fetcher = GitFetcher::new(dir.path().join("cache"));
    let url = bare.to_string_lossy().to_string();

    let result = fetcher
        .fetch(&url, &GitRef::DefaultBranch, None)
        .expect("fetch should succeed");

    assert_eq!(result.commit.as_str(), expected_sha);
    assert!(result.path.join("SKILL.md").exists());
    assert!(result.path.join("scripts/helper.sh").exists());
}

#[test]
fn fetch_cached_returns_same_path() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, _) = make_skill_repo(dir.path());
    let fetcher = GitFetcher::new(dir.path().join("cache"));
    let url = bare.to_string_lossy().to_string();

    let r1 = fetcher.fetch(&url, &GitRef::DefaultBranch, None).unwrap();
    let r2 = fetcher
        .fetch(&url, &GitRef::Commit(r1.commit.clone()), Some(&r1.commit))
        .unwrap();

    assert_eq!(r1.commit, r2.commit);
    assert_eq!(r1.path, r2.path);
}

#[test]
fn fetch_nonexistent_ref_fails() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, _) = make_skill_repo(dir.path());
    let fetcher = GitFetcher::new(dir.path().join("cache"));
    let url = bare.to_string_lossy().to_string();

    let result = fetcher.fetch(&url, &GitRef::Branch("no-such-branch".into()), None);
    assert!(result.is_err());
}

#[test]
fn evict_removes_all_cached_data() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, _) = make_skill_repo(dir.path());
    let fetcher = GitFetcher::new(dir.path().join("cache"));
    let url = bare.to_string_lossy().to_string();

    let result = fetcher.fetch(&url, &GitRef::DefaultBranch, None).unwrap();
    assert!(fetcher.is_cached(&url, &result.commit));

    fetcher.evict(&url).unwrap();
    assert!(!fetcher.is_cached(&url, &result.commit));
}

#[test]
fn skill_at_repo_root_has_valid_skill_md() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, _) = make_skill_repo(dir.path());
    let fetcher = GitFetcher::new(dir.path().join("cache"));
    let url = bare.to_string_lossy().to_string();

    let result = fetcher.fetch(&url, &GitRef::DefaultBranch, None).unwrap();

    let skill_md = result.path.join("SKILL.md");
    assert!(skill_md.exists(), "SKILL.md should exist at checkout root");

    let content = fs_err::read_to_string(&skill_md).unwrap();
    assert!(content.contains("name: test-skill"));
    assert!(content.contains("description: a test skill"));
}

#[test]
fn skill_in_subdirectory() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, _) = make_multiskill_repo(dir.path());
    let fetcher = GitFetcher::new(dir.path().join("cache"));
    let url = bare.to_string_lossy().to_string();

    let result = fetcher.fetch(&url, &GitRef::DefaultBranch, None).unwrap();

    // root should NOT have SKILL.md
    assert!(!result.path.join("SKILL.md").exists());

    // subdirectory skills should be accessible
    let frontend = result.path.join("skills/frontend-design/SKILL.md");
    assert!(frontend.exists());
    assert!(
        fs_err::read_to_string(&frontend)
            .unwrap()
            .contains("name: frontend-design")
    );

    let backend = result.path.join("skills/backend-api/SKILL.md");
    assert!(backend.exists());
}

#[test]
fn nonexistent_subdirectory_has_no_skill_md() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, _) = make_multiskill_repo(dir.path());
    let fetcher = GitFetcher::new(dir.path().join("cache"));
    let url = bare.to_string_lossy().to_string();

    let result = fetcher.fetch(&url, &GitRef::DefaultBranch, None).unwrap();
    assert!(!result.path.join("skills/nonexistent/SKILL.md").exists());
}

#[test]
fn lock_resolves_git_skill_at_root() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, expected_sha) = make_skill_repo(dir.path());
    let project = dir.path().join("project");
    fs_err::create_dir_all(&project).unwrap();

    let url = bare.to_string_lossy().to_string();
    let manifest_str = format!(
        r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"

[skills.my-skill]
source = {{ git = "{url}" }}
"#
    );
    fs_err::write(project.join("theta.toml"), &manifest_str).unwrap();

    let manifest: theta_schema::ThetaManifest = toml::from_str(&manifest_str).unwrap();
    let git_cache = dir.path().join("git-cache");
    fs_err::create_dir_all(&git_cache).unwrap();

    let lock = theta_lock::build_lock(&manifest, manifest_str.as_bytes(), &project, &git_cache)
        .expect("build_lock should succeed");

    let entry = lock
        .skills
        .get("my-skill")
        .expect("my-skill should be locked");
    match &entry.source {
        theta_lock::LockedSource::Git {
            git,
            resolved_commit,
            subdirectory,
            ..
        } => {
            assert_eq!(git, &url);
            assert_eq!(resolved_commit.as_str(), expected_sha);
            assert!(subdirectory.is_none());
        }
        other => panic!("expected LockedSource::Git, got {other:?}"),
    }
    assert!(entry.content_hash.to_string().starts_with("sha256:"));
}

#[test]
fn lock_resolves_git_skill_in_subdirectory() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, expected_sha) = make_multiskill_repo(dir.path());
    let project = dir.path().join("project");
    fs_err::create_dir_all(&project).unwrap();

    let url = bare.to_string_lossy().to_string();
    let manifest_str = format!(
        r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"

[skills.frontend-design]
source = {{ git = "{url}", subdirectory = "skills/frontend-design" }}
"#
    );
    fs_err::write(project.join("theta.toml"), &manifest_str).unwrap();

    let manifest: theta_schema::ThetaManifest = toml::from_str(&manifest_str).unwrap();
    let git_cache = dir.path().join("git-cache");
    fs_err::create_dir_all(&git_cache).unwrap();

    let lock = theta_lock::build_lock(&manifest, manifest_str.as_bytes(), &project, &git_cache)
        .expect("build_lock should succeed");

    let entry = lock
        .skills
        .get("frontend-design")
        .expect("frontend-design should be locked");
    match &entry.source {
        theta_lock::LockedSource::Git {
            resolved_commit,
            subdirectory,
            ..
        } => {
            assert_eq!(resolved_commit.as_str(), expected_sha);
            assert_eq!(subdirectory.as_deref(), Some("skills/frontend-design"));
        }
        other => panic!("expected LockedSource::Git, got {other:?}"),
    }
    assert!(entry.content_hash.to_string().starts_with("sha256:"));
}

#[test]
fn lock_fails_when_skill_md_missing() {
    let dir = tempfile::tempdir().unwrap();
    let (bare, _) = make_multiskill_repo(dir.path());
    let project = dir.path().join("project");
    fs_err::create_dir_all(&project).unwrap();

    let url = bare.to_string_lossy().to_string();
    let manifest_str = format!(
        r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"

[skills.nope]
source = {{ git = "{url}", subdirectory = "does-not-exist" }}
"#
    );
    fs_err::write(project.join("theta.toml"), &manifest_str).unwrap();

    let manifest: theta_schema::ThetaManifest = toml::from_str(&manifest_str).unwrap();
    let git_cache = dir.path().join("git-cache");
    fs_err::create_dir_all(&git_cache).unwrap();

    let result = theta_lock::build_lock(&manifest, manifest_str.as_bytes(), &project, &git_cache);
    let errors = result.unwrap_err();
    assert!(
        matches!(&errors[0], theta_lock::BuildError::GitFileNotFound { file, .. } if file.contains("SKILL.md")),
        "expected GitFileNotFound mentioning SKILL.md, got: {:?}",
        errors[0]
    );
}

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn fetch_real_github_repo() {
    let dir = tempfile::tempdir().unwrap();
    let fetcher = GitFetcher::new(dir.path().join("git"));
    let url = "https://github.com/anthropics/prompt-eng-interactive-tutorial";

    let result = fetcher
        .fetch(url, &GitRef::DefaultBranch, None)
        .expect("fetch from github should work");

    assert_eq!(result.commit.as_str().len(), 40);
    assert!(result.path.join("README.md").exists());
}

const VERCEL_SKILLS_REPO: &str = "https://github.com/vercel-labs/agent-skills";

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn vercel_agent_skills_has_skill_md_in_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    let fetcher = GitFetcher::new(dir.path().join("git"));

    let result = fetcher
        .fetch(VERCEL_SKILLS_REPO, &GitRef::DefaultBranch, None)
        .expect("fetch vercel-labs/agent-skills");

    // the repo root should NOT have SKILL.md (it's a multi-skill repo)
    assert!(
        !result.path.join("SKILL.md").exists(),
        "repo root should not contain SKILL.md"
    );

    // skills live under skills/<name>/SKILL.md
    let web_design = result.path.join("skills/web-design-guidelines/SKILL.md");
    assert!(
        web_design.exists(),
        "web-design-guidelines/SKILL.md should exist"
    );
    let content = fs_err::read_to_string(&web_design).unwrap();
    assert!(
        content.contains("name:") && content.contains("description:"),
        "SKILL.md should have frontmatter with name and description"
    );

    let composition = result.path.join("skills/composition-patterns/SKILL.md");
    assert!(
        composition.exists(),
        "composition-patterns/SKILL.md should exist"
    );
}

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn lock_resolves_vercel_skill_with_subdirectory() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs_err::create_dir_all(&project).unwrap();

    let manifest_str = format!(
        r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"

[skills.web-design-guidelines]
source = {{ git = "{VERCEL_SKILLS_REPO}", subdirectory = "skills/web-design-guidelines" }}
"#
    );
    fs_err::write(project.join("theta.toml"), &manifest_str).unwrap();

    let manifest: theta_schema::ThetaManifest = toml::from_str(&manifest_str).unwrap();
    let git_cache = dir.path().join("git-cache");
    fs_err::create_dir_all(&git_cache).unwrap();

    let lock = theta_lock::build_lock(&manifest, manifest_str.as_bytes(), &project, &git_cache)
        .expect("build_lock for vercel skill should succeed");

    let entry = lock
        .skills
        .get("web-design-guidelines")
        .expect("web-design-guidelines should be locked");
    match &entry.source {
        theta_lock::LockedSource::Git {
            git,
            resolved_commit,
            subdirectory,
            ..
        } => {
            assert_eq!(git, VERCEL_SKILLS_REPO);
            assert_eq!(resolved_commit.as_str().len(), 40);
            assert_eq!(
                subdirectory.as_deref(),
                Some("skills/web-design-guidelines")
            );
        }
        other => panic!("expected LockedSource::Git, got {other:?}"),
    }
    assert!(entry.content_hash.to_string().starts_with("sha256:"));
}

// non-conformant: repos that do NOT have SKILL.md should fail lock

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn lock_fails_for_repo_without_skill_md_at_root() {
    // anthropics/prompt-eng-interactive-tutorial has no SKILL.md
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs_err::create_dir_all(&project).unwrap();

    let manifest_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"

[skills.bad-skill]
source = { git = "https://github.com/anthropics/prompt-eng-interactive-tutorial" }
"#;
    fs_err::write(project.join("theta.toml"), manifest_str).unwrap();

    let manifest: theta_schema::ThetaManifest = toml::from_str(manifest_str).unwrap();
    let git_cache = dir.path().join("git-cache");
    fs_err::create_dir_all(&git_cache).unwrap();

    let result = theta_lock::build_lock(&manifest, manifest_str.as_bytes(), &project, &git_cache);
    let errors = result.unwrap_err();
    assert!(
        matches!(&errors[0], theta_lock::BuildError::GitFileNotFound { file, .. } if file.contains("SKILL.md")),
        "expected GitFileNotFound mentioning SKILL.md, got: {:?}",
        errors[0]
    );
}

#[test]
#[cfg_attr(not(feature = "online-tests"), ignore)]
fn lock_fails_for_wrong_subdirectory_in_real_repo() {
    // vercel-labs/agent-skills exists but skills/does-not-exist does not
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs_err::create_dir_all(&project).unwrap();

    let manifest_str = format!(
        r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"

[skills.fake-skill]
source = {{ git = "{VERCEL_SKILLS_REPO}", subdirectory = "skills/does-not-exist" }}
"#
    );
    fs_err::write(project.join("theta.toml"), &manifest_str).unwrap();

    let manifest: theta_schema::ThetaManifest = toml::from_str(&manifest_str).unwrap();
    let git_cache = dir.path().join("git-cache");
    fs_err::create_dir_all(&git_cache).unwrap();

    let result = theta_lock::build_lock(&manifest, manifest_str.as_bytes(), &project, &git_cache);
    let errors = result.unwrap_err();
    assert!(
        matches!(&errors[0], theta_lock::BuildError::GitFileNotFound { file, .. } if file.contains("SKILL.md")),
        "expected GitFileNotFound mentioning SKILL.md, got: {:?}",
        errors[0]
    );
}
