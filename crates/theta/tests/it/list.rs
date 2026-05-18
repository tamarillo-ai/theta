use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn rules_empty_project() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.list("rules").assert().success();
}

#[test]
fn rules_shows_added_rule() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("safety").assert().success();

    ctx.list("rules")
        .assert()
        .success()
        .stdout(predicate::str::contains("safety"));
}

#[test]
fn skills_shows_added_skill() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("skill")
        .args(["code-review", "--no-sync"])
        .assert()
        .success();

    ctx.list("skills")
        .assert()
        .success()
        .stdout(predicate::str::contains("code-review"));
}

#[test]
fn tools_shows_added_tool() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("tool")
        .args(["my-tool", "--command", "echo hello"])
        .assert()
        .success();

    ctx.list("tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("my-tool"));
}

#[test]
fn subagents_shows_added_subagent() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("subagent")
        .args(["researcher", "--description", "searches the web"])
        .assert()
        .success();

    ctx.list("subagents")
        .assert()
        .success()
        .stdout(predicate::str::contains("researcher"));
}

#[test]
fn store_lists_registered_items() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    // register a rule into the store first
    ctx.add("rule").arg("safety").assert().success();
    let rule_path = ctx.temp_dir.child("instructions/rules/safety.md");
    fs_err::write(rule_path.path(), "Always be safe.\n").unwrap();
    ctx.register("rule").arg("safety").assert().success();

    ctx.list("store")
        .assert()
        .success()
        .stdout(predicate::str::contains("safety"));
}
