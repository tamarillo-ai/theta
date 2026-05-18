use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn shows_placeholder_on_fresh_project() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.describe().assert().success();
}

#[test]
fn set_updates_manifest() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.describe()
        .arg("--set")
        .arg("A test agent for unit testing")
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("A test agent for unit testing"));
}

#[test]
fn describe_positional_sets_description() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.describe()
        .arg("A positional description")
        .assert()
        .success();

    ctx.describe()
        .assert()
        .success()
        .stdout(predicate::str::contains("A positional description"));
}

#[test]
fn describe_positional_and_set_conflict() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.describe()
        .arg("positional")
        .arg("--set")
        .arg("also set")
        .assert()
        .failure();
}

#[test]
fn describe_shows_rules_flag() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    // --rules should succeed even with no rules
    ctx.describe().arg("--rules").assert().success();
}
