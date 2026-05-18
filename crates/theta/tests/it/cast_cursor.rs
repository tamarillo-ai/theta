//! Cursor cast e2e tests.
//!
//! Each test runs `theta cast from cursor` and/or `theta cast to cursor`
//! and asserts on workspace state. Tests are grouped by resource type.

use theta_test::checkers::cursor;
use theta_test::test_context;

// rules

#[test]
fn rules_always_apply_round_trip() {
    let ctx = test_context!().with_fixture("cursor/rules/always-apply");
    let original = ctx.snapshot_original();

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    cursor::assert_rule_equal(
        &original.join(".cursor/rules/safety.mdc"),
        &ctx.path(".cursor/rules/safety.mdc"),
    );
}

#[test]
fn rules_glob_pattern_round_trip() {
    let ctx = test_context!().with_fixture("cursor/rules/glob-pattern");
    let original = ctx.snapshot_original();

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    cursor::assert_rule_equal(
        &original.join(".cursor/rules/react.mdc"),
        &ctx.path(".cursor/rules/react.mdc"),
    );
}

#[test]
fn rules_leading_star_glob_preserved() {
    // `globs: *.ts` is invalid YAML but valid .mdc - the cursor-specific
    // parser handles it as a line-based key:value format.
    let ctx = test_context!().with_fixture("cursor/rules/leading-star-glob");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("*.ts"),
        "leading-star glob should be preserved by mdc parser, got:\n{toml}"
    );
    assert!(
        toml.contains("apply = \"glob\""),
        "should be glob-scoped, got:\n{toml}"
    );
}

#[test]
fn rules_always_apply_with_globs_round_trip() {
    // ref: https://cursor.com/docs/rules#rule-anatomy - alwaysApply + globs coexist
    let ctx = test_context!().with_fixture("cursor/rules/always-apply-with-globs");
    let original = ctx.snapshot_original();

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    cursor::assert_rule_equal(
        &original.join(".cursor/rules/scoped.mdc"),
        &ctx.path(".cursor/rules/scoped.mdc"),
    );
}

#[test]
fn rules_path_qualified_flattened_on_round_trip() {
    // cursor has flat rule dirs - a theta manifest with "review/pr-review" should
    // produce `.cursor/rules/review-pr-review.mdc` (flattened with `-`).
    let ctx = test_context!().with_fixture("cursor/rules/path-qualified");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    // import sees `review-pr-review.mdc` and creates flat rule name
    assert!(
        toml.contains("review-pr-review"),
        "cursor import should use flat name from filename, got:\n{toml}"
    );

    // round-trip back
    ctx.cast_to("cursor").arg("--force").assert().success();

    let rule_path = ctx.path(".cursor/rules/review-pr-review.mdc");
    assert!(rule_path.exists(), "flattened rule file should exist");
    let content = fs_err::read_to_string(&rule_path).unwrap();
    assert!(content.contains("reviewing a pull request"));
}

// mcp

#[test]
fn mcp_basic_round_trip() {
    let ctx = test_context!().with_fixture("cursor/mcp/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    cursor::assert_mcp_equal(
        &original.join(".cursor/mcp.json"),
        &ctx.path(".cursor/mcp.json"),
    );
}

// agents

#[test]
fn agent_basic_round_trip() {
    // ref: https://cursor.com/docs/subagents
    let ctx = test_context!().with_fixture("cursor/agents/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    cursor::assert_agent_equal(
        &original.join(".cursor/agents/verifier.md"),
        &ctx.path(".cursor/agents/verifier.md"),
    );
    cursor::assert_agent_equal(
        &original.join(".cursor/agents/impact-checker.md"),
        &ctx.path(".cursor/agents/impact-checker.md"),
    );
}

// import-only checks

#[test]
fn import_produces_theta_toml() {
    let ctx = test_context!().with_fixture("cursor/rules/always-apply");

    ctx.cast_from("cursor").arg("--force").assert().success();

    assert!(ctx.exists("theta.toml"));
    let toml = ctx.read_file("theta.toml");
    assert!(toml.contains("[theta]"));
    assert!(toml.contains("[agent]"));
}

#[test]
fn import_always_apply_maps_correctly() {
    let ctx = test_context!().with_fixture("cursor/rules/always-apply");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("apply = \"always\""),
        "alwaysApply: true should map to apply = always, got:\n{toml}"
    );
}

#[test]
fn import_glob_preserves_pattern() {
    let ctx = test_context!().with_fixture("cursor/rules/glob-pattern");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("src/components/**/*.tsx"),
        "glob pattern should be preserved in theta.toml, got:\n{toml}"
    );
}

#[test]
fn import_leading_star_glob_fields_and_body_preserved() {
    // `globs: *.ts` is invalid YAML but the mdc parser handles it.
    let ctx = test_context!().with_fixture("cursor/rules/leading-star-glob");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("*.ts"),
        "leading-star glob should be preserved, got:\n{toml}"
    );
    let rule_body = ctx.read_file("rules/typescript.md");
    assert!(
        rule_body.contains("strict TypeScript"),
        "rule body must be extracted, got:\n{rule_body}"
    );
}

#[test]
fn import_always_apply_with_globs_preserves_both() {
    let ctx = test_context!().with_fixture("cursor/rules/always-apply-with-globs");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("apply = \"always\""),
        "alwaysApply: true should win, got:\n{toml}"
    );
    assert!(
        toml.contains("src/**/*.ts"),
        "glob patterns should be preserved alongside always, got:\n{toml}"
    );
}

#[test]
fn import_agent_readonly_preserved() {
    let ctx = test_context!().with_fixture("cursor/agents/basic");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("readonly = true"),
        "readonly should be preserved, got:\n{toml}"
    );
}

#[test]
fn rules_comma_separated_globs_round_trip() {
    let ctx = test_context!().with_fixture("cursor/rules/comma-separated-globs");
    let original = ctx.snapshot_original();

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    cursor::assert_rule_equal(
        &original.join(".cursor/rules/multi.mdc"),
        &ctx.path(".cursor/rules/multi.mdc"),
    );
}

#[test]
fn import_comma_separated_globs_all_preserved() {
    let ctx = test_context!().with_fixture("cursor/rules/comma-separated-globs");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("src/db/*.ts"),
        "first pattern missing:\n{toml}"
    );
    assert!(
        toml.contains("src/middleware/*.ts"),
        "second pattern missing:\n{toml}"
    );
    assert!(
        toml.contains("src/lib/*.ts"),
        "third pattern missing:\n{toml}"
    );
}

// warnings

#[test]
fn mcp_wrong_root_key_emits_warning() {
    // ref: https://cursor.com/docs/mcp#using-mcpjson - root key must be "mcpServers"
    let ctx = test_context!().with_fixture("cursor/mcp/wrong-root-key");

    let output = ctx.cast_from("cursor").arg("--force").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mcpServers"),
        "should warn about missing mcpServers key, got:\n{stderr}"
    );
}

#[test]
fn mcp_wrong_root_key_does_not_import_tools() {
    let ctx = test_context!().with_fixture("cursor/mcp/wrong-root-key");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        !toml.contains("[tools"),
        "wrong-key mcp.json should not produce [tools], got:\n{toml}"
    );
}

#[test]
fn colon_in_description_fully_imported() {
    // `description: Go patterns: Repository` has `: ` inside the value.
    // the mdc parser handles this - everything after first colon is the value.
    let ctx = test_context!().with_fixture("cursor/rules/invalid-yaml-colon");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[instructions.rules.patterns]"),
        "rule should be imported, got:\n{toml}"
    );
    assert!(
        toml.contains("internal/**/*.go"),
        "globs should be imported by mdc parser, got:\n{toml}"
    );
    assert!(
        toml.contains("Go design patterns"),
        "description with colon should be preserved, got:\n{toml}"
    );
    let rule_body = ctx.read_file("rules/patterns.md");
    assert!(
        rule_body.contains("Repository pattern"),
        "rule body must be preserved, got:\n{rule_body}"
    );
}

// hooks

#[test]
fn hooks_import_round_trips() {
    let ctx = test_context!().with_fixture("cursor/hooks/basic");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[harness.cursor.hooks]"),
        "hooks should be imported into harness config, got:\n{toml}"
    );
    assert!(
        toml.contains("format.sh"),
        "hook command should be preserved, got:\n{toml}"
    );

    // round-trip back
    ctx.cast_to("cursor").arg("--force").assert().success();

    let hooks = ctx.read_file(".cursor/hooks.json");
    assert!(
        hooks.contains("format.sh"),
        "hook command should survive round-trip, got:\n{hooks}"
    );
    assert!(
        hooks.contains("audit.sh"),
        "stop hook should survive round-trip, got:\n{hooks}"
    );
}

// skills

#[test]
fn skills_import_produces_skills_section() {
    let ctx = test_context!().with_fixture("cursor/skills/basic");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[skills.deploy-app]"),
        "skill should be imported, got:\n{toml}"
    );
}

#[test]
fn skills_round_trip_preserves_content() {
    let ctx = test_context!().with_fixture("cursor/skills/basic");

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    let skill_md = ctx.read_file(".cursor/skills/deploy-app/SKILL.md");
    assert!(
        skill_md.contains("deploy-app"),
        "SKILL.md should survive round-trip, got:\n{skill_md}"
    );
    assert!(
        ctx.path(".cursor/skills/deploy-app/scripts/deploy.sh")
            .exists(),
        "skill scripts should survive round-trip"
    );
}

// mcp with extras (auth, etc.)

#[test]
fn mcp_extras_round_trip() {
    let ctx = test_context!().with_fixture("cursor/mcp/with-extras");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[tools.sqlite]"),
        "sqlite tool should be imported, got:\n{toml}"
    );
    assert!(
        toml.contains("oauth-api") || toml.contains("[tools.oauth-api]"),
        "oauth-api tool should be imported, got:\n{toml}"
    );
    // auth is an unmodeled extra - should land in harness config
    assert!(
        toml.contains("mcp_extras") || toml.contains("auth"),
        "auth field should be preserved as extra, got:\n{toml}"
    );

    // round-trip back
    ctx.cast_to("cursor").arg("--force").assert().success();

    let mcp = ctx.read_file(".cursor/mcp.json");
    assert!(
        mcp.contains("sqlite"),
        "sqlite should survive round-trip, got:\n{mcp}"
    );
    assert!(
        mcp.contains("abc123") || mcp.contains("CLIENT_ID"),
        "auth extras should be merged back, got:\n{mcp}"
    );
}

// combined: full configuration

#[test]
fn combined_full_config_imports_all_surfaces() {
    let ctx = test_context!().with_fixture("cursor/combined/full-config");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    // identity
    assert!(
        toml.contains("full-config-agent") || toml.contains("[agent]"),
        "agent should be imported, got:\n{toml}"
    );
    // rules
    assert!(
        toml.contains("[instructions.rules.typescript]"),
        "typescript rule should be imported, got:\n{toml}"
    );
    assert!(
        toml.contains("[instructions.rules.safety]"),
        "safety rule should be imported, got:\n{toml}"
    );
    // tools
    assert!(
        toml.contains("[tools.sqlite]"),
        "sqlite tool should be imported, got:\n{toml}"
    );
    // hooks
    assert!(
        toml.contains("[harness.cursor.hooks]"),
        "hooks should be imported, got:\n{toml}"
    );
    // skills
    assert!(
        toml.contains("[skills.deploy]"),
        "deploy skill should be imported, got:\n{toml}"
    );
    // subagents
    assert!(
        toml.contains("reviewer"),
        "reviewer subagent should be imported, got:\n{toml}"
    );
    assert!(
        toml.contains("implementer"),
        "implementer subagent should be imported, got:\n{toml}"
    );
}

#[test]
fn combined_full_config_round_trips_all_surfaces() {
    let ctx = test_context!().with_fixture("cursor/combined/full-config");

    ctx.cast_from("cursor").arg("--force").assert().success();
    ctx.cast_to("cursor").arg("--force").assert().success();

    // rules survived
    let ts_rule = ctx.read_file(".cursor/rules/typescript.mdc");
    assert!(
        ts_rule.contains("strict TypeScript"),
        "typescript rule body should survive, got:\n{ts_rule}"
    );
    assert!(
        ts_rule.contains("globs:"),
        "typescript globs should be emitted, got:\n{ts_rule}"
    );
    let safety_rule = ctx.read_file(".cursor/rules/safety.mdc");
    assert!(
        safety_rule.contains("alwaysApply: true"),
        "safety alwaysApply should survive, got:\n{safety_rule}"
    );

    // hooks survived
    let hooks = ctx.read_file(".cursor/hooks.json");
    assert!(
        hooks.contains("format.sh"),
        "hooks should survive round-trip, got:\n{hooks}"
    );

    // mcp survived
    let mcp = ctx.read_file(".cursor/mcp.json");
    assert!(
        mcp.contains("sqlite"),
        "mcp tools should survive, got:\n{mcp}"
    );

    // agents survived
    let reviewer = ctx.read_file(".cursor/agents/reviewer.md");
    assert!(
        reviewer.contains("code reviewer"),
        "reviewer agent should survive, got:\n{reviewer}"
    );

    // skills survived
    let skill = ctx.read_file(".cursor/skills/deploy/SKILL.md");
    assert!(
        skill.contains("deploy"),
        "skill should survive, got:\n{skill}"
    );
}
// cross-read

#[test]
fn cross_read_imports_agents_md_into_system_prompt() {
    let ctx = test_context!().with_fixture("cursor/cross-read/basic");

    ctx.cast_from("cursor")
        .arg("--force")
        .arg("--cross-read")
        .assert()
        .success();

    let system = ctx.read_file(theta_static::SYSTEM_FILE_NAME);
    assert!(
        system.contains("help with code"),
        "system prompt should contain native .cursor/rules/system.md content:\n{system}"
    );
    assert!(
        system.contains("project conventions"),
        "system prompt should contain AGENTS.md content via cross-read:\n{system}"
    );
}

#[test]
fn cross_read_imports_claude_agents_as_subagents() {
    let ctx = test_context!().with_fixture("cursor/cross-read/basic");

    ctx.cast_from("cursor")
        .arg("--force")
        .arg("--cross-read")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("researcher"),
        "should import .claude/agents/researcher.md as subagent:\n{toml}"
    );
    assert!(
        toml.contains("deep research specialist"),
        "should preserve description from .claude/agents/ frontmatter:\n{toml}"
    );
}

#[test]
fn cross_read_imports_codex_agents_as_subagents() {
    let ctx = test_context!().with_fixture("cursor/cross-read/basic");

    ctx.cast_from("cursor")
        .arg("--force")
        .arg("--cross-read")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("planner"),
        "should import .codex/agents/planner.toml as subagent:\n{toml}"
    );
    assert!(
        toml.contains("breaks down complex tasks"),
        "should preserve description from codex TOML:\n{toml}"
    );
    // developer_instructions should be extracted to a prompt file
    let prompt = theta_static::ThetaProjectLayout::subagent_prompt_rel("planner");
    assert!(
        ctx.exists(&prompt),
        "codex developer_instructions should be extracted to {prompt}"
    );
    let content = ctx.read_file(&prompt);
    assert!(
        content.contains("step-by-step"),
        "extracted prompt should contain developer_instructions:\n{content}"
    );
}

#[test]
fn cross_read_emits_cursor_diagnostics() {
    let ctx = test_context!().with_fixture("cursor/cross-read/basic");

    let output = ctx
        .cast_from("cursor")
        .arg("--force")
        .arg("--cross-read")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cross-read"),
        "should emit cross-read diagnostics:\n{stderr}"
    );
}

#[test]
fn without_cross_read_cursor_skips_agents_md() {
    let ctx = test_context!().with_fixture("cursor/cross-read/basic");

    ctx.cast_from("cursor").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        !toml.contains("project conventions"),
        "without --cross-read, AGENTS.md content should NOT appear:\n{toml}"
    );
    assert!(
        !toml.contains("researcher"),
        "without --cross-read, .claude/agents/ should NOT be imported:\n{toml}"
    );
    assert!(
        !toml.contains("planner"),
        "without --cross-read, .codex/agents/ should NOT be imported:\n{toml}"
    );
}
