use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn help_shows_usage() {
    let ctx = test_context!();
    ctx.command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn version_flag() {
    let ctx = test_context!();
    ctx.command()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("theta"));
}
