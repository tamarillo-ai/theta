use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn creates_manifest() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success()
        .stderr(predicate::str::contains("initialized"));

    let manifest = ctx.temp_dir.child("theta.toml");
    manifest.assert(predicates::path::exists());
    let content = fs_err::read_to_string(manifest.path()).unwrap();
    assert!(content.contains("test-agent"));
}

#[test]
fn refuses_overwrite() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn bare_creates_manifest_with_default_name() {
    let ctx = test_context!();
    ctx.init().assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("[theta]"));
    assert!(manifest.contains("[agent]"));
}

#[test]
fn with_custom_name() {
    let ctx = test_context!();
    ctx.init()
        .args(["--name", "my-custom-agent"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("my-custom-agent"));
}

#[test]
fn force_without_from_fails() {
    let ctx = test_context!();
    ctx.init().arg("--force").assert().failure().code(2);
}

#[test]
fn from_nonexistent_agent_fails() {
    let ctx = test_context!();
    ctx.init()
        .args(["--from", "nonexistent-agent-xyz"])
        .assert()
        .failure();
}
