//! Claude Code cast e2e tests.
//!
//! Each test runs `theta cast from claude-code` and/or `theta cast to claude-code`
//! against fixtures under `test/cast-fixtures/claude-code/` and asserts on
//! workspace state. Tests are grouped by resource type.

use theta_test::checkers::{body, claude_code};
use theta_test::test_context;

// system prompt

#[test]
fn system_prompt_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/system-prompt/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    body::assert_body_equal(&original.join("CLAUDE.md"), &ctx.path("CLAUDE.md"));
}

#[test]
fn system_prompt_alternate_location() {
    let ctx = test_context!().with_fixture("claude-code/system-prompt/alternate");

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("system = \"system.md\""),
        "alternate .claude/CLAUDE.md should be imported as system prompt"
    );

    ctx.cast_to("claude-code").arg("--force").assert().success();

    // cast writes to root CLAUDE.md (canonical location)
    assert!(
        ctx.exists("CLAUDE.md"),
        "cast should write CLAUDE.md at project root"
    );
}

#[test]
fn system_prompt_not_emitted_when_absent() {
    let ctx = test_context!().with_fixture("claude-code/settings/permissions");

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    assert!(
        !ctx.exists("CLAUDE.md"),
        "CLAUDE.md must not be emitted when system prompt is absent"
    );
}

// rules

#[test]
fn rules_with_paths_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/rules/with-paths");
    let original = ctx.snapshot_original();

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    for rule in [
        "language.md",
        "coding-style.md",
        "git-commit.md",
        "agents.md",
    ] {
        let rel = format!(".claude/rules/{rule}");
        claude_code::assert_rule_equal(&original.join(&rel), &ctx.path(&rel));
    }
}

// agents

#[test]
fn agents_basic_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/agents/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    for agent in ["code-reviewer.md", "debugger.md"] {
        let rel = format!(".claude/agents/{agent}");
        claude_code::assert_agent_equal(&original.join(&rel), &ctx.path(&rel));
    }
}

#[test]
fn agents_no_description() {
    let ctx = test_context!().with_fixture("claude-code/agents/no-description");

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[[subagents]]"),
        "subagent should be imported even without description"
    );
    // the subagent entry should NOT have a description key since the
    // source frontmatter didn't have one and Subagent.description is
    // Option<String>. check that no `description =` line appears between
    // [[subagents]] and the next section.
    let after_subagent = toml.split("[[subagents]]").nth(1).unwrap_or("");
    let subagent_block = after_subagent.split("\n[").next().unwrap_or("");
    assert!(
        !subagent_block.contains("description"),
        "description should not be fabricated when absent in source: {subagent_block}"
    );

    ctx.cast_to("claude-code").arg("--force").assert().success();

    // the agent file should exist after round-trip
    assert!(
        ctx.exists(".claude/agents/searcher.md"),
        "agent file should exist after round-trip"
    );
}

#[test]
fn agents_with_extras_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/agents/with-extras");

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    // extras should land somewhere under [harness.claude_code]
    assert!(
        toml.contains("permissionMode"),
        "permissionMode should round-trip as extra"
    );
    assert!(toml.contains("memory"), "memory should round-trip as extra");
    assert!(toml.contains("effort"), "effort should round-trip as extra");
    assert!(toml.contains("color"), "color should round-trip as extra");

    ctx.cast_to("claude-code").arg("--force").assert().success();

    let cast = ctx.read_file(".claude/agents/security-checker.md");
    assert!(
        cast.contains("permissionMode"),
        "permissionMode should appear in cast frontmatter"
    );
    assert!(
        cast.contains("memory"),
        "memory should appear in cast frontmatter"
    );
}

// settings

#[test]
fn settings_permissions_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/settings/permissions");
    let original = ctx.snapshot_original();

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    claude_code::assert_settings_equal(
        &original.join(".claude/settings.json"),
        &ctx.path(".claude/settings.json"),
    );
}

#[test]
fn settings_hooks_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/settings/hooks");
    let original = ctx.snapshot_original();

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    claude_code::assert_settings_equal(
        &original.join(".claude/settings.json"),
        &ctx.path(".claude/settings.json"),
    );
}

// MCP

#[test]
fn mcp_basic_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/mcp/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    claude_code::assert_mcp_equal(&original.join(".mcp.json"), &ctx.path(".mcp.json"));
}

#[test]
fn mcp_with_extras_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/mcp/with-extras");

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    // extras (oauth, alwaysLoad) should land in [harness.claude_code.tool.<name>]
    assert!(
        toml.contains("[harness.claude_code.tool.remote-api]") || toml.contains("tool.remote-api"),
        "MCP server extras should be under [harness.claude_code.tool.<name>]"
    );

    ctx.cast_to("claude-code").arg("--force").assert().success();

    claude_code::assert_mcp_equal(
        &ctx.snapshot_original().join(".mcp.json"),
        &ctx.path(".mcp.json"),
    );
}

// skills

#[test]
fn skills_basic_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/skills/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();
    ctx.cast_to("claude-code").arg("--force").assert().success();

    body::assert_file_identical(
        &original.join(".claude/skills/lint-check/SKILL.md"),
        &ctx.path(".claude/skills/lint-check/SKILL.md"),
    );
}

// combined

#[test]
fn combined_everything_round_trip() {
    let ctx = test_context!().with_fixture("claude-code/combined/everything");

    ctx.cast_from("claude-code")
        .arg("--force")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    assert!(toml.contains("[instructions]"), "should have instructions");
    assert!(toml.contains("[[subagents]]"), "should have subagents");
    assert!(toml.contains("[tools"), "should have tools");
    assert!(
        toml.contains("[harness.claude_code]"),
        "should have harness config"
    );

    ctx.cast_to("claude-code").arg("--force").assert().success();

    // verify all surface types exist in output
    assert!(ctx.exists("CLAUDE.md"), "system prompt");
    assert!(ctx.exists(".claude/settings.json"), "settings");
    assert!(ctx.exists(".mcp.json"), "mcp");
    assert!(ctx.exists(".claude/skills/deploy/SKILL.md"), "skills");
}
