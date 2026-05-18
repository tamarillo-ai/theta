use assert_fs::fixture::PathChild;
use theta_test::test_context;

#[test]
fn directory_flag_works() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.command()
        .args(["--directory", ".", "check", "--schema-only"])
        .assert()
        .success();
}

#[test]
fn directory_flag_nonexistent_fails() {
    let ctx = test_context!();
    ctx.command()
        .args(["--directory", "/nonexistent", "check"])
        .assert()
        .failure();
}

#[test]
fn manifest_flag_explicit_path() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.command()
        .args(["--manifest", "theta.toml", "check", "--schema-only"])
        .assert()
        .success();
}

#[test]
fn manifest_flag_nonexistent_fails() {
    let ctx = test_context!();
    ctx.command()
        .args(["--manifest", "/nonexistent/theta.toml", "check"])
        .assert()
        .failure();
}

#[test]
fn instructions_dir_override() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args(["--instructions-dir", "custom-inst", "my-rule"])
        .assert()
        .success();

    let rule_file = ctx.temp_dir.child("custom-inst/rules/my-rule.md");
    assert!(
        rule_file.path().exists(),
        "rule should be in custom instructions dir"
    );
}

#[test]
fn rules_dir_override() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args(["--rules-dir", "custom-rules", "my-rule"])
        .assert()
        .success();

    let rule_file = ctx.temp_dir.child("instructions/custom-rules/my-rule.md");
    assert!(
        rule_file.path().exists(),
        "rule should be in custom rules dir"
    );
}
