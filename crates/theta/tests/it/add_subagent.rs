use assert_fs::fixture::{FileWriteStr, PathChild};
use theta_test::test_context;

//  happy path

#[test]
fn inline_registers_in_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args(["researcher", "--description", "searches the web"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("[[subagents]]"));
    assert!(manifest.contains("researcher"));
}

//  valid argument combinations

#[test]
fn with_model() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args([
            "planner",
            "--description",
            "Plans tasks",
            "--model",
            "claude-sonnet-4-20250514",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("claude-sonnet-4-20250514"));
}

#[test]
fn add_subagent_with_prompt_path() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    let prompt = ctx.temp_dir.child("subagents/writer.md");
    fs_err::create_dir_all(prompt.path().parent().unwrap()).unwrap();
    fs_err::write(prompt.path(), "You write clearly.").unwrap();

    ctx.add("subagent")
        .args([
            "writer",
            "--description",
            "Writes docs",
            "--prompt-path",
            "subagents/writer.md",
        ])
        .assert()
        .success();
}

#[test]
fn add_subagent_with_tools() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("tool")
        .args(["mytool", "--command", "echo hi"])
        .assert()
        .success();

    ctx.add("subagent")
        .args(["coder", "--description", "Writes code", "--tools", "mytool"])
        .assert()
        .success();
}

#[test]
fn add_subagent_with_skills() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args([
            "coder",
            "--description",
            "Writes code",
            "--skills",
            "review,deploy",
        ])
        .assert()
        .success();
}

#[test]
fn add_subagent_with_agent_ref() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    let scout_manifest = ctx.temp_dir.child("agents/scout/theta.toml");
    fs_err::create_dir_all(scout_manifest.path().parent().unwrap()).unwrap();
    scout_manifest
        .write_str(
            r#"[theta]
schema = "2026-04"

[agent]
name = "scout"
description = "Scouts codebases"
version = "0.1.0"
model = "claude-sonnet-4-20250514"

[instructions]
system = "instructions/system.md"
"#,
        )
        .unwrap();
    let sys = ctx.temp_dir.child("agents/scout/instructions/system.md");
    fs_err::create_dir_all(sys.path().parent().unwrap()).unwrap();
    fs_err::write(sys.path(), "You scout code.").unwrap();

    ctx.add("subagent")
        .args(["scout", "--agent-ref", "agents/scout/theta.toml"])
        .assert()
        .success();
}

//  invalid combinations

#[test]
fn add_subagent_without_description_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent").arg("broken").assert().failure();
}

#[test]
fn add_subagent_duplicate_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args(["researcher", "--description", "v1"])
        .assert()
        .success();

    ctx.add("subagent")
        .args(["researcher", "--description", "v2"])
        .assert()
        .failure();
}

#[test]
fn add_subagent_agent_ref_conflicts_with_description() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args(["x", "--agent-ref", "foo.toml", "--description", "conflict"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_subagent_agent_ref_conflicts_with_model() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args(["x", "--agent-ref", "foo.toml", "--model", "conflict"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_subagent_agent_ref_conflicts_with_tools() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args(["x", "--agent-ref", "foo.toml", "--tools", "conflict"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_subagent_agent_ref_nonexistent_fails() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args(["x", "--agent-ref", "/nonexistent/file.toml"])
        .assert()
        .failure();
}
#[test]
fn description_only_creates_no_prompt_file() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("subagent")
        .args([
            "lightweight",
            "--description",
            "A lightweight helper",
            "--description-only",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("lightweight"));
    // no prompt file should be created
    assert!(
        !ctx.temp_dir
            .child("subagents/lightweight.md")
            .path()
            .exists()
    );
}
