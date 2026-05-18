use theta_test::test_context;

#[test]
fn on_project_without_subagents() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.tree().assert().success();
}

#[test]
fn json_on_project_without_subagents() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.tree()
        .args(["--output-format", "json"])
        .assert()
        .success();
}
