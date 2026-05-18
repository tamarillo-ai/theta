use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use theta_test::test_context;

#[test]
fn rule_into_system_store() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("safety").assert().success();

    let rule_path = ctx.temp_dir.child("instructions/rules/safety.md");
    fs_err::write(rule_path.path(), "Always prioritize user safety.\n").unwrap();

    ctx.register("rule").arg("safety").assert().success();

    ctx.data_dir
        .child("store")
        .assert(predicates::path::exists());
}

#[test]
fn rule_nonexistent_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.register("rule").arg("nonexistent").assert().failure();
}
