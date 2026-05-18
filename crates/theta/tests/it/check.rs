use assert_fs::fixture::{FileWriteStr, PathChild};
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn on_fresh_project() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.check().assert().success();
}

#[test]
fn schema_only() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.check().arg("--schema-only").assert().success();
}

#[test]
fn check_skip_materialization() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.check().arg("--skip-materialization").assert().success();
}

#[test]
fn check_schema_only_and_skip_materialization_conflict() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.check()
        .arg("--schema-only")
        .arg("--skip-materialization")
        .assert()
        .failure();
}

#[test]
fn check_no_manifest_fails() {
    let ctx = test_context!();
    // no init - no manifest
    ctx.check().assert().failure();
}

#[test]
fn check_corrupt_manifest_fails() {
    let ctx = test_context!();
    let manifest = ctx.temp_dir.child("theta.toml");
    manifest.write_str("garbage{{{").unwrap();

    ctx.check()
        .assert()
        .failure()
        .stderr(predicate::str::contains("parse").or(predicate::str::contains("invalid")));
}
