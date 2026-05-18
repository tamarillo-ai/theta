use assert_fs::assert::PathAssert;
use assert_fs::fixture::{FileWriteStr, PathChild};
use predicates::prelude::*;
use theta_test::test_context;

//  happy path

#[test]
fn scaffold_creates_directory() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args(["code-review", "--no-sync"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("[skills.code-review]"));

    ctx.temp_dir
        .child("skills/code-review")
        .assert(predicates::path::exists());
    ctx.temp_dir
        .child("skills/code-review/SKILL.md")
        .assert(predicates::path::exists());
}

#[test]
fn rejects_invalid_name() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args(["INVALID_NAME", "--no-sync"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not a valid skill name"));
}

//  valid argument combinations

#[test]
fn with_description() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args([
            "deploy",
            "--description",
            "Deploy applications",
            "--no-sync",
        ])
        .assert()
        .success();
}

#[test]
fn add_skill_with_path_existing_dir() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    let skill_dir = ctx.temp_dir.child("custom-skills/analysis");
    fs_err::create_dir_all(skill_dir.path()).unwrap();
    let skill_md = skill_dir.child("SKILL.md");
    skill_md
        .write_str("---\nname: analysis\ndescription: Analyze code\n---\n# analysis\n")
        .unwrap();

    ctx.add("skill")
        .args(["analysis", "--path", "custom-skills/analysis"])
        .assert()
        .success();
}

#[test]
fn add_skill_with_git() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args([
            "web-design",
            "--git",
            "https://github.com/vercel-labs/agent-skills",
            "--subdirectory",
            "skills/web-design-guidelines",
            "--no-sync",
        ])
        .assert()
        .success();
}

#[test]
fn add_skill_with_branch() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args([
            "patterns",
            "--git",
            "https://github.com/vercel-labs/agent-skills",
            "--branch",
            "main",
            "--subdirectory",
            "skills/composition-patterns",
            "--no-sync",
        ])
        .assert()
        .success();
}

//  invalid combinations

#[test]
fn add_skill_rejects_uppercase() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["UPPER", "--no-sync"])
        .assert()
        .failure();
}

#[test]
fn add_skill_rejects_consecutive_hyphens() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["bad--skill", "--no-sync"])
        .assert()
        .failure();
}

#[test]
fn add_skill_duplicate_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["review", "--no-sync"])
        .assert()
        .success();
    ctx.add("skill")
        .args(["review", "--no-sync"])
        .assert()
        .failure();
}

#[test]
fn add_skill_path_conflicts_with_git() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["x", "--path", "foo", "--git", "https://x"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_skill_with_tag() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args([
            "pinned",
            "--git",
            "https://github.com/vercel-labs/agent-skills",
            "--tag",
            "v1.0.0",
            "--no-sync",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("tag = \"v1.0.0\""));
}

#[test]
fn add_skill_with_rev() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args([
            "frozen",
            "--git",
            "https://github.com/org/skills",
            "--rev",
            "abc1234",
            "--no-sync",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("rev = \"abc1234\""));
}

#[test]
fn add_skill_branch_and_tag_conflict() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args([
            "x",
            "--git",
            "https://example.com",
            "--branch",
            "main",
            "--tag",
            "v1.0",
            "--no-sync",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_skill_branch_and_rev_conflict() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args([
            "x",
            "--git",
            "https://example.com",
            "--branch",
            "main",
            "--rev",
            "abc123",
            "--no-sync",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_skill_branch_requires_git() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["x", "--branch", "main", "--no-sync"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_skill_path_nonexistent_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["x", "--path", "/nonexistent/dir"])
        .assert()
        .failure();
}

#[test]
fn add_skill_path_without_skill_md_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    let empty_dir = ctx.temp_dir.child("empty-skill");
    fs_err::create_dir_all(empty_dir.path()).unwrap();

    ctx.add("skill")
        .args(["x", "--path", "empty-skill"])
        .assert()
        .failure();
}
