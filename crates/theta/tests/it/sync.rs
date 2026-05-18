use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn sync_materializes_files() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("Be helpful.")
        .assert()
        .success();

    ctx.command()
        .args(["add", "rule", "safe"])
        .arg("--content")
        .arg("Be safe.")
        .assert()
        .success();

    ctx.sync().assert().success();

    let theta_dir = ctx.temp_dir.child(".theta");
    theta_dir.assert(predicates::path::exists());

    let system = ctx.temp_dir.child(".theta/system.md");
    system.assert(predicates::path::exists());
    let content = fs_err::read_to_string(system.path()).unwrap();
    assert!(content.contains("Be helpful"));

    let rule = ctx.temp_dir.child(".theta/rules/safe.md");
    rule.assert(predicates::path::exists());
    let content = fs_err::read_to_string(rule.path()).unwrap();
    assert!(content.contains("Be safe"));
}

#[test]
fn sync_is_idempotent() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("Be helpful.")
        .assert()
        .success();

    ctx.sync().assert().success();

    ctx.sync()
        .assert()
        .success()
        .stderr(predicate::str::contains("up to date"));
}

#[test]
fn sync_force_rematerializes() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("Be helpful.")
        .assert()
        .success();

    ctx.sync().assert().success();

    ctx.sync().arg("--force").assert().success();
}

#[test]
fn sync_materializes_path_qualified_rule() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.command()
        .args(["add", "rule", "review/pr-review", "--content", "Check PRs."])
        .assert()
        .success();

    ctx.sync().assert().success();

    let rule = ctx.temp_dir.child(".theta/rules/review/pr-review.md");
    rule.assert(predicates::path::exists());
    let content = fs_err::read_to_string(rule.path()).unwrap();
    assert!(content.contains("Check PRs"));
}

#[test]
fn sync_fails_when_materialized_skill_content_is_invalid() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args(["analysis", "--no-sync"])
        .assert()
        .success();

    let skill = ctx.temp_dir.child("skills/analysis/SKILL.md");
    fs_err::write(skill.path(), "---\nname: analysis\n---\n# analysis\n").unwrap();

    ctx.sync()
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "materialized but content validation failed",
        ))
        .stderr(predicate::str::contains(
            "SKILL.md frontmatter is missing required `description` field",
        ));
}
