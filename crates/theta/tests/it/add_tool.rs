use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

//  happy path

#[test]
fn stdio_registers_in_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args(["my-tool", "--command", "npx -y @example/tool"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("[tools.my-tool]"));
    assert!(manifest.contains("npx"));
}

#[test]
fn http_registers_in_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args(["web-tool", "--url", "https://example.com/mcp"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("[tools.web-tool]"));
    assert!(manifest.contains("https://example.com/mcp"));
}

//  valid argument combinations

#[test]
fn with_env_vars() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args([
            "db",
            "--command",
            "npx @db/mcp",
            "--env",
            "DB_HOST=localhost",
            "--env",
            "DB_PORT=5432",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("DB_HOST"));
    assert!(manifest.contains("DB_PORT"));
}

#[test]
fn add_tool_with_args() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args([
            "fs",
            "--command",
            "npx @mcp/fs",
            "--args",
            "/tmp",
            "--args",
            "/home",
        ])
        .assert()
        .success();
}

#[test]
fn add_tool_with_disabled() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args(["experimental", "--command", "npx @exp/tool", "--disabled"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("enabled = false"));
}

#[test]
fn add_tool_http_with_headers() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args([
            "auth-api",
            "--url",
            "https://api.example.com/mcp",
            "--header",
            "Authorization=Bearer token",
            "--header",
            "X-Region=us-east-1",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("Authorization"));
    assert!(manifest.contains("X-Region"));
}

#[test]
fn add_tool_full_combo() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args([
            "mega",
            "--command",
            "uvx mega-tool",
            "--env",
            "KEY=val",
            "--args=--verbose",
            "--disabled",
        ])
        .assert()
        .success();
}

//  invalid combinations

#[test]
fn add_tool_no_transport_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .arg("broken")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--command").or(predicate::str::contains("--url")));
}

#[test]
fn add_tool_command_and_url_conflict() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args(["broken", "--command", "test", "--url", "http://x"])
        .assert()
        .failure();
}

#[test]
fn add_tool_duplicate_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args(["ctx7", "--command", "npx test"])
        .assert()
        .success();

    ctx.add("tool")
        .args(["ctx7", "--command", "test"])
        .assert()
        .failure();
}

#[test]
fn add_tool_rejects_uppercase_name() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args(["UPPER", "--command", "test"])
        .assert()
        .failure();
}

#[test]
fn add_tool_rejects_underscore_name() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("tool")
        .args(["under_score", "--command", "test"])
        .assert()
        .failure();
}
