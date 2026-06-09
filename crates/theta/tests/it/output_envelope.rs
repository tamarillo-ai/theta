//! End-to-end contract tests for `--output-format json`.
//!
//! For every leaf verb that can be driven from a tmpdir without external
//! state (no network, no system store), assert:
//!
//! 1. stdout is exactly one parseable JSON document
//! 2. the document has the canonical envelope keys
//!    (`verb`, `status`, `diagnostics`, `data`)
//! 3. exit code matches `status` (`ok`/`noop` -> 0, `error` -> 1)
//!
//! These tests are the contract theta-py codegen relies on.

use serde_json::Value;
use theta_test::TestContext;
use theta_test::test_context;

/// Run a verb in JSON mode, expect success, parse the envelope.
fn run_ok(ctx: &TestContext, argv: &[&str]) -> Value {
    let mut cmd = ctx.command();
    cmd.args(["--output-format", "json"]).args(argv);
    let out = cmd.assert().success().get_output().clone();
    parse_envelope(&out.stdout, &["ok", "noop"])
}

/// Run a verb in JSON mode, expect exit code 1 and `status: "error"`.
fn run_err(ctx: &TestContext, argv: &[&str]) -> Value {
    let mut cmd = ctx.command();
    cmd.args(["--output-format", "json"]).args(argv);
    let out = cmd.assert().failure().code(1).get_output().clone();
    let env = parse_envelope(&out.stdout, &["error"]);
    assert!(
        env["diagnostics"].as_array().is_some_and(|d| !d.is_empty()),
        "error envelope should carry at least one diagnostic",
    );
    env
}

fn parse_envelope(stdout: &[u8], allowed_statuses: &[&str]) -> Value {
    let s = std::str::from_utf8(stdout).expect("stdout was not valid UTF-8");
    let env: Value = serde_json::from_str(s).expect("stdout was not a single JSON document");
    let obj = env.as_object().expect("envelope is not a JSON object");
    for key in ["verb", "status", "diagnostics", "data"] {
        assert!(obj.contains_key(key), "envelope missing key {key:?}: {env}");
    }
    assert!(env["verb"].is_array(), "verb must be an array");
    assert!(env["status"].is_string(), "status must be a string");
    assert!(
        env["diagnostics"].is_array(),
        "diagnostics must be an array"
    );
    let status = env["status"].as_str().unwrap();
    assert!(
        allowed_statuses.contains(&status),
        "expected status in {allowed_statuses:?}, got {status:?}",
    );
    env
}

fn init_project(ctx: &TestContext) {
    run_ok(ctx, &["init", "--name", "env-test"]);
}

#[test]
fn init_emits_ok_envelope() {
    let ctx = test_context!();
    let env = run_ok(&ctx, &["init", "--name", "env-test"]);
    assert_eq!(env["verb"], serde_json::json!(["init"]));
    assert_eq!(env["data"]["source"], serde_json::json!("scaffold"));
}

#[test]
fn init_already_exists_emits_error_envelope() {
    let ctx = test_context!();
    init_project(&ctx);
    let env = run_err(&ctx, &["init"]);
    assert_eq!(env["verb"], serde_json::json!(["init"]));
}

#[test]
fn check_emits_envelope_with_diagnostics() {
    let ctx = test_context!();
    init_project(&ctx);
    let env = run_ok(&ctx, &["check"]);
    assert_eq!(env["verb"], serde_json::json!(["check"]));
    assert!(env["data"]["valid"].as_bool().is_some());
}

#[test]
fn migrate_emits_noop_on_current_schema() {
    let ctx = test_context!();
    init_project(&ctx);
    let env = run_ok(&ctx, &["migrate"]);
    assert_eq!(env["status"], serde_json::json!("noop"));
}

#[test]
fn lock_then_lock_again_emits_noop() {
    let ctx = test_context!();
    init_project(&ctx);

    let env = run_ok(&ctx, &["lock"]);
    assert_eq!(env["status"], serde_json::json!("ok"));
    assert_eq!(env["data"]["wrote"], serde_json::json!(true));

    let env = run_ok(&ctx, &["lock"]);
    assert_eq!(env["status"], serde_json::json!("noop"));
    assert_eq!(env["data"]["wrote"], serde_json::json!(false));
}

#[test]
fn add_then_rm_rule_emit_envelopes() {
    let ctx = test_context!();
    init_project(&ctx);

    let env = run_ok(&ctx, &["add", "rule", "safety"]);
    assert_eq!(env["verb"], serde_json::json!(["add", "rule"]));
    assert_eq!(env["data"]["entity"], serde_json::json!("rule"));
    assert_eq!(env["data"]["name"], serde_json::json!("safety"));

    let env = run_ok(&ctx, &["rm", "rule", "safety", "--no-sync"]);
    assert_eq!(env["verb"], serde_json::json!(["rm", "rule"]));
    assert_eq!(env["data"]["entity"], serde_json::json!("rule"));
    assert_eq!(env["data"]["name"], serde_json::json!("safety"));
}

#[test]
fn list_rules_emits_envelope_with_kind() {
    let ctx = test_context!();
    init_project(&ctx);
    let env = run_ok(&ctx, &["list", "rules"]);
    assert_eq!(env["verb"], serde_json::json!(["list", "rules"]));
    assert_eq!(env["data"]["kind"], serde_json::json!("rules"));
}

#[test]
fn tree_emits_envelope_with_root_node() {
    let ctx = test_context!();
    init_project(&ctx);
    let env = run_ok(&ctx, &["tree"]);
    assert_eq!(env["verb"], serde_json::json!(["tree"]));
    assert!(env["data"]["tree"]["name"].is_string());
}

#[test]
fn describe_emits_envelope_in_read_mode() {
    let ctx = test_context!();
    init_project(&ctx);
    let env = run_ok(&ctx, &["describe"]);
    assert_eq!(env["verb"], serde_json::json!(["describe"]));
    assert_eq!(env["data"]["mode"], serde_json::json!("read"));
}

/// Regression: internal sync/lock calls used to leak their own JSON envelopes
/// to stdout when invoked from add/rm. Verify the outer verb emits exactly
/// one document on stdout.
#[test]
fn stdout_contains_only_one_json_document() {
    let ctx = test_context!();
    init_project(&ctx);

    let mut cmd = ctx.command();
    cmd.args(["--output-format", "json", "add", "rule", "safety"]);
    let out = cmd.assert().success().get_output().clone();
    let stdout = std::str::from_utf8(&out.stdout).unwrap();

    let mut de = serde_json::Deserializer::from_str(stdout).into_iter::<Value>();
    assert!(
        de.next().is_some(),
        "stdout should contain one JSON document"
    );
    assert!(
        de.next().is_none(),
        "stdout should contain only one JSON document, found more: {stdout}",
    );
}
