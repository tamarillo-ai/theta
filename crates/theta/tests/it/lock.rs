use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn lock_creates_lockfile() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("You are helpful.")
        .assert()
        .success();

    ctx.lock().assert().success();

    let lockfile = ctx.temp_dir.child("theta.lock");
    let content = fs_err::read_to_string(lockfile.path()).unwrap();
    assert!(content.contains("manifest_hash"));
    assert!(content.contains("system"));
}

#[test]
fn lock_is_idempotent() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("You help.")
        .assert()
        .success();

    ctx.lock().assert().success();

    // second lock should be a no-op
    ctx.lock()
        .assert()
        .success()
        .stderr(predicate::str::contains("up to date"));
}

#[test]
fn lock_force_re_resolves() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("You help.")
        .assert()
        .success();

    ctx.lock().assert().success();

    ctx.lock()
        .arg("--force")
        .assert()
        .success()
        .stderr(predicate::str::contains("wrote"));
}

#[test]
fn lock_preserves_path_qualified_rule_key() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.command()
        .args(["add", "rule", "review/pr-review", "--content", "Check PRs."])
        .assert()
        .success();

    ctx.lock().assert().success();

    let lockfile = ctx.temp_dir.child("theta.lock");
    let content = fs_err::read_to_string(lockfile.path()).unwrap();
    assert!(
        content.contains("\"review/pr-review\""),
        "lockfile should preserve path-qualified rule key, got:\n{content}"
    );
    assert!(
        content.contains("instructions/rules/review/pr-review.md"),
        "lockfile should have subdirectory path"
    );
}
