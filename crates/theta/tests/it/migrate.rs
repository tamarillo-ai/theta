use theta_test::test_context;

#[test]
fn migrate_no_op_on_current_schema() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command().arg("migrate").assert().success();
}

#[test]
fn migrate_dry_run() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["migrate", "--dry-run"])
        .assert()
        .success();
}

#[test]
fn migrate_no_manifest_fails() {
    let ctx = test_context!();
    // no init - no manifest
    ctx.command().arg("migrate").assert().failure();
}
