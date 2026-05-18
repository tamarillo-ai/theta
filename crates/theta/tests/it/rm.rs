use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn rule_removes_from_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("safety").assert().success();

    ctx.rm("rule").arg("safety").assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(!manifest.contains("safety"));
}

#[test]
fn nonexistent_rule_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.rm("rule").arg("nonexistent").assert().failure().stderr(
        predicate::str::contains("not registered").or(predicate::str::contains("not found")),
    );
}

#[test]
fn tool_removes_from_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("tool")
        .args(["my-tool", "--command", "echo hello"])
        .assert()
        .success();

    ctx.rm("tool").arg("my-tool").assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(!manifest.contains("my-tool"));
}

#[test]
fn skill_removes_from_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["code-review", "--no-sync"])
        .assert()
        .success();

    ctx.rm("skill").arg("code-review").assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(!manifest.contains("code-review"));
}

#[test]
fn subagent_removes_from_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("subagent")
        .args(["researcher", "--description", "searches the web"])
        .assert()
        .success();

    ctx.rm("subagent").arg("researcher").assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(!manifest.contains("researcher"));
}

#[test]
fn system_removes_from_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("system").assert().success();

    ctx.rm("system").assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(!manifest.contains("[instructions]"));
}

#[test]
fn rule_with_delete_removes_source_file() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("safety").assert().success();

    let rule_file = ctx.temp_dir.child("instructions/rules/safety.md");
    rule_file.assert(predicates::path::exists());

    ctx.rm("rule")
        .args(["safety", "--delete"])
        .assert()
        .success();

    rule_file.assert(predicates::path::missing());
}
#[test]
fn store_unregisters_rule() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule").arg("safety").assert().success();
    let rule_path = ctx.temp_dir.child("instructions/rules/safety.md");
    fs_err::write(rule_path.path(), "Always be safe.\n").unwrap();
    ctx.register("rule").arg("safety").assert().success();

    ctx.list("store")
        .assert()
        .success()
        .stdout(predicate::str::contains("safety"));

    ctx.rm("store").args(["rule", "safety"]).assert().success();

    ctx.list("store")
        .assert()
        .success()
        .stdout(predicate::str::contains("safety").not());
}
