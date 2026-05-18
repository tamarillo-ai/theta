//! Copilot cast e2e tests.
//!
//! Each test runs `theta cast from copilot` and/or `theta cast to copilot`
//! and asserts on workspace state. Tests are grouped by resource type.

use theta_test::checkers::{body, copilot};
use theta_test::test_context;

// system-prompt

#[test]
fn system_prompt_round_trip() {
    let ctx = test_context!().with_fixture("copilot/system-prompt/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    body::assert_body_equal(
        &original.join(".github/copilot-instructions.md"),
        &ctx.path(".github/copilot-instructions.md"),
    );
}

#[test]
fn system_prompt_not_emitted_when_absent() {
    let ctx = test_context!().with_files(&[(
        ".github/instructions/placeholder.instructions.md",
        "---\napplyTo: \"**\"\n---\n\nplaceholder rule\n",
    )]);

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    assert!(
        !ctx.exists(".github/copilot-instructions.md"),
        "copilot-instructions.md should not be emitted when system prompt is absent"
    );
}

// rules

#[test]
fn rules_glob_round_trip() {
    let ctx = test_context!().with_fixture("copilot/rules/glob-patterns");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    for rule in [
        "general-coding.instructions.md",
        "typescript-react.instructions.md",
    ] {
        let rel = format!(".github/instructions/{rule}");
        copilot::assert_rule_equal(&original.join(&rel), &ctx.path(&rel));
    }
}

#[test]
fn rules_subdirectory_round_trip() {
    let ctx = test_context!().with_fixture("copilot/rules/subdirectory");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_rule_equal(
        &original.join(".github/instructions/review/pr-review.instructions.md"),
        &ctx.path(".github/instructions/review/pr-review.instructions.md"),
    );
}

#[test]
fn rules_model_decision_round_trip() {
    let ctx = test_context!().with_fixture("copilot/rules/model-decision");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();

    // verify description is preserved in theta.toml
    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("description"),
        "description should be in theta.toml"
    );

    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_rule_equal(
        &original.join(".github/instructions/cypress-best-practices.instructions.md"),
        &ctx.path(".github/instructions/cypress-best-practices.instructions.md"),
    );
}

#[test]
fn rules_multi_pattern_glob_round_trip() {
    let ctx = test_context!().with_fixture("copilot/rules/multi-pattern-glob");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    // multi-pattern applyTo with spaces should round-trip semantically
    copilot::assert_rule_equal(
        &original.join(".github/instructions/article-pages.instructions.md"),
        &ctx.path(".github/instructions/article-pages.instructions.md"),
    );
}

#[test]
fn rule_no_frontmatter_becomes_manual() {
    let ctx = test_context!().with_files(&[(
        ".github/instructions/bare.instructions.md",
        "no frontmatter, just content.\n",
    )]);

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("apply = \"manual\""),
        "rule without frontmatter should get apply = manual, got:\n{toml}"
    );
}

#[test]
fn rule_always_apply() {
    let ctx = test_context!().with_files(&[(
        ".github/instructions/global.instructions.md",
        "---\napplyTo: \"**\"\n---\n\nalways-on rule content\n",
    )]);

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("apply = \"always\""),
        "applyTo: ** should map to apply = always, got:\n{toml}"
    );
}

#[test]
fn rule_with_description_becomes_model_decision() {
    let ctx = test_context!().with_files(&[(
        ".github/instructions/smart.instructions.md",
        "---\ndescription: apply when editing tests\n---\n\ntest conventions\n",
    )]);

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("apply = \"model-decision\""),
        "rule with description and no applyTo should get model-decision, got:\n{toml}"
    );
}

// agents

#[test]
fn agents_extras_round_trip() {
    let ctx = test_context!().with_fixture("copilot/agents/with-extras");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    for agent in ["docs-researcher.agent.md", "python-reviewer.agent.md"] {
        let rel = format!(".github/agents/{agent}");
        if ctx.exists(&rel) {
            copilot::assert_agent_equal(&original.join(&rel), &ctx.path(&rel));
        }
    }
}

#[test]
fn agents_with_handoffs_round_trip() {
    let ctx = test_context!().with_fixture("copilot/agents/with-handoffs");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_agent_equal_ignoring(
        &original.join(".github/agents/ba.agent.md"),
        &ctx.path(".github/agents/ba.agent.md"),
        &["name"],
    );
    let cast = ctx.read_file(".github/agents/ba.agent.md");
    assert!(
        cast.contains("name:"),
        "cast should emit name: in agent frontmatter"
    );
}

#[test]
fn agent_no_frontmatter_stays_clean() {
    let ctx = test_context!().with_fixture("copilot/agents/no-frontmatter");

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    let cast = ctx.read_file(".github/agents/architect.agent.md");
    assert!(
        !cast.contains("imported from .github/agents/"),
        "default description should not be injected into agent without frontmatter"
    );
}

#[test]
fn agent_no_frontmatter_body_preserved() {
    let ctx = test_context!().with_fixture("copilot/agents/no-frontmatter");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    body::assert_body_equal(
        &original.join(".github/agents/architect.agent.md"),
        &ctx.path(".github/agents/architect.agent.md"),
    );
}

// skills

#[test]
fn skill_basic_round_trip() {
    let ctx = test_context!().with_fixture("copilot/skills/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_skill_equal(
        &original.join(".github/skills/create-section"),
        &ctx.path(".github/skills/create-section"),
    );
}

#[test]
fn skill_with_references_round_trip() {
    let ctx = test_context!().with_fixture("copilot/skills/with-references");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_skill_equal(
        &original.join(".github/skills/building-native-ui"),
        &ctx.path(".github/skills/building-native-ui"),
    );
}

// settings

#[test]
fn settings_mixed_keys_semantically_equal() {
    let ctx = test_context!().with_fixture("copilot/settings/mixed-keys");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_settings_equal(
        &original.join(".vscode/settings.json"),
        &ctx.path(".vscode/settings.json"),
    );
}
#[test]
fn settings_chat_keys_round_trip() {
    let ctx = test_context!().with_fixture("copilot/settings/chat-keys");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("chat.tools.terminal.autoApprove")
            || toml.contains("chat.agentFilesLocations"),
        "chat.* keys should be imported into theta.toml"
    );

    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_settings_equal(
        &original.join(".vscode/settings.json"),
        &ctx.path(".vscode/settings.json"),
    );
}

#[test]
fn settings_only_non_copilot_keys_preserved() {
    let ctx = test_context!().with_fixture("copilot/settings/only-non-copilot");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        !toml.contains("chat.") && !toml.contains("github.copilot"),
        "non-copilot settings should not appear in theta.toml"
    );

    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_settings_equal(
        &original.join(".vscode/settings.json"),
        &ctx.path(".vscode/settings.json"),
    );
}

#[test]
fn settings_github_copilot_keys_round_trip() {
    let ctx = test_context!().with_fixture("copilot/settings/github-copilot-keys");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("github.copilot.chat.codeGeneration.useInstructionFiles"),
        "github.copilot.* keys should be imported"
    );

    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_settings_equal(
        &original.join(".vscode/settings.json"),
        &ctx.path(".vscode/settings.json"),
    );
}

// mcp

#[test]
fn mcp_basic_server_round_trip() {
    let ctx = test_context!().with_fixture("copilot/mcp/basic-server");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_mcp_equal(
        &original.join(".vscode/mcp.json"),
        &ctx.path(".vscode/mcp.json"),
    );
}

// combined

#[test]
fn combined_everything_round_trip() {
    let ctx = test_context!().with_fixture("copilot/combined/everything");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    // system prompt
    body::assert_body_equal(
        &original.join(".github/copilot-instructions.md"),
        &ctx.path(".github/copilot-instructions.md"),
    );

    // rules
    copilot::assert_rule_equal(
        &original.join(".github/instructions/article-pages.instructions.md"),
        &ctx.path(".github/instructions/article-pages.instructions.md"),
    );

    // agents
    for agent in ["designer.agent.md", "developer.agent.md"] {
        let rel = format!(".github/agents/{agent}");
        if ctx.exists(&rel) {
            copilot::assert_agent_equal_ignoring(&original.join(&rel), &ctx.path(&rel), &["name"]);
        }
    }

    // skills
    copilot::assert_skill_equal(
        &original.join(".github/skills/ship-checker"),
        &ctx.path(".github/skills/ship-checker"),
    );

    // CLAUDE.md untouched
    if original.join("CLAUDE.md").exists() {
        body::assert_file_identical(&original.join("CLAUDE.md"), &ctx.path("CLAUDE.md"));
    }
}

#[test]
fn combined_japanese_frontmatter_round_trip() {
    let ctx = test_context!().with_fixture("copilot/combined/japanese");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    body::assert_body_equal(
        &original.join(".github/copilot-instructions.md"),
        &ctx.path(".github/copilot-instructions.md"),
    );

    copilot::assert_rule_equal(
        &original.join(".github/instructions/python.instructions.md"),
        &ctx.path(".github/instructions/python.instructions.md"),
    );

    let agent = "docs-researcher.agent.md";
    let rel = format!(".github/agents/{agent}");
    if ctx.exists(&rel) {
        copilot::assert_agent_equal(&original.join(&rel), &ctx.path(&rel));
    }
}

//  import-only checks (no round-trip)

#[test]
fn import_produces_theta_toml() {
    let ctx = test_context!().with_fixture("copilot/system-prompt/basic");

    ctx.cast_from("copilot").arg("--force").assert().success();

    assert!(ctx.exists("theta.toml"), "theta.toml should be created");
    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[theta]"),
        "theta.toml should have [theta] section"
    );
    assert!(
        toml.contains("[agent]"),
        "theta.toml should have [agent] section"
    );
}

#[test]
fn import_extracts_rule_body() {
    let ctx = test_context!().with_fixture("copilot/rules/glob-patterns");

    ctx.cast_from("copilot").arg("--force").assert().success();

    assert!(
        ctx.exists("rules/general-coding.md"),
        "rule body should be extracted"
    );
    assert!(
        ctx.exists("rules/typescript-react.md"),
        "rule body should be extracted"
    );
}

#[test]
fn import_extracts_subdirectory_rule() {
    let ctx = test_context!().with_fixture("copilot/rules/subdirectory");

    ctx.cast_from("copilot").arg("--force").assert().success();

    assert!(
        ctx.exists("rules/review/pr-review.md"),
        "subdirectory rule should be extracted with path structure"
    );

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("\"review/pr-review\""),
        "path-qualified rule name should appear in theta.toml"
    );
}

// agents: model field

#[test]
fn agent_with_model_round_trip() {
    let ctx = test_context!().with_fixture("copilot/agents/with-model");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();

    // model should appear in theta.toml
    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("model ="),
        "model should be preserved in theta.toml"
    );

    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_agent_equal_ignoring(
        &original.join(".github/agents/python-architect.agent.md"),
        &ctx.path(".github/agents/python-architect.agent.md"),
        &["name"],
    );
}

// agents: many (scale + slug)

#[test]
fn agents_many_import_succeeds() {
    let ctx = test_context!().with_fixture("copilot/agents/many");

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    let count = toml.matches("[[subagents]]").count();
    assert!(count >= 4, "should import multiple agents, got {count}");
}

// mcp: multiple servers

#[test]
fn mcp_multiple_servers_round_trip() {
    let ctx = test_context!().with_fixture("copilot/mcp/multiple-servers");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    copilot::assert_mcp_equal(
        &original.join(".vscode/mcp.json"),
        &ctx.path(".vscode/mcp.json"),
    );
}

// settings: JSONC comments

#[test]
fn settings_jsonc_comments_semantically_equal() {
    let ctx = test_context!().with_fixture("copilot/settings/jsonc-comments");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    // JSONC comments are stripped (known limitation) but all keys preserved
    copilot::assert_settings_equal(
        &original.join(".vscode/settings.json"),
        &ctx.path(".vscode/settings.json"),
    );
}

// ignored: non-standard files

#[test]
fn non_standard_files_in_instructions_ignored() {
    let ctx = test_context!().with_fixture("copilot/ignored/non-standard-in-instructions");

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    // README.md should NOT appear as a rule
    assert!(
        !toml.contains("README"),
        "non-.instructions.md files should not be imported as rules"
    );
    // .instructions.md files SHOULD appear
    assert!(
        toml.contains("[instructions.rules"),
        "valid .instructions.md files should be imported"
    );
}

// cross-harness files

#[test]
fn claude_md_survives_round_trip() {
    let ctx = test_context!().with_fixture("copilot/combined/everything");
    let original = ctx.snapshot_original();

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    if original.join("CLAUDE.md").exists() {
        assert!(ctx.exists("CLAUDE.md"), "CLAUDE.md should survive");
        body::assert_file_identical(&original.join("CLAUDE.md"), &ctx.path("CLAUDE.md"));
    }
}

// inline edge cases

#[test]
fn rule_body_with_yaml_like_content() {
    let ctx = test_context!().with_files(&[(
        ".github/instructions/tricky.instructions.md",
        "---\napplyTo: \"**\"\n---\n\n# heading\n\n---\n\nsome content after a horizontal rule\n",
    )]);

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    // the  in the body should NOT be treated as frontmatter
    let cast = ctx.read_file(".github/instructions/tricky.instructions.md");
    assert!(
        cast.contains("horizontal rule"),
        "body content after  should survive"
    );
}

#[test]
fn empty_settings_json_survives() {
    let ctx = test_context!().with_files(&[
        (".vscode/settings.json", "{}"),
        (".github/copilot-instructions.md", "hello\n"),
    ]);

    ctx.cast_from("copilot").arg("--force").assert().success();
    ctx.cast_to("copilot").arg("--force").assert().success();

    // empty settings should not crash and file should still exist
    assert!(ctx.exists(".vscode/settings.json"));
}

#[test]
fn prompts_hint_emitted() {
    let ctx = test_context!().with_files(&[
        (".github/prompts/test.prompt.md", "test prompt\n"),
        (".github/copilot-instructions.md", "hello\n"),
    ]);

    let output = ctx.cast_from("copilot").arg("--force").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("prompt") || stderr.contains("hint"),
        "should emit a hint about prompts not being modeled"
    );
}
// cross-read

#[test]
fn cross_read_imports_agents_md_and_claude_md() {
    let ctx = test_context!().with_fixture("copilot/cross-read/basic");

    ctx.cast_from("copilot")
        .arg("--force")
        .arg("--cross-read")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[instructions]"),
        "should have instructions section:\n{toml}"
    );

    let system = ctx.read_file(theta_static::SYSTEM_FILE_NAME);
    assert!(
        system.contains("helpful assistant"),
        "system prompt should contain native content:\n{system}"
    );
    assert!(
        system.contains("functional style"),
        "system prompt should contain AGENTS.md content:\n{system}"
    );
    assert!(
        system.contains("TypeScript"),
        "system prompt should contain CLAUDE.md content:\n{system}"
    );
}

#[test]
fn cross_read_imports_claude_rules() {
    let ctx = test_context!().with_fixture("copilot/cross-read/basic");

    ctx.cast_from("copilot")
        .arg("--force")
        .arg("--cross-read")
        .assert()
        .success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("claude-python-style"),
        "should import .claude/rules/ as rules with claude- prefix:\n{toml}"
    );
    let rule_path = theta_static::ThetaProjectLayout::rule_rel("claude-python-style");
    assert!(
        ctx.exists(&rule_path),
        "rule file should be extracted at {rule_path}"
    );
    let rule_content = ctx.read_file(&rule_path);
    assert!(
        rule_content.contains("type hints"),
        "rule content should contain the original rule body:\n{rule_content}"
    );
}

#[test]
fn cross_read_emits_diagnostics() {
    let ctx = test_context!().with_fixture("copilot/cross-read/basic");

    let output = ctx
        .cast_from("copilot")
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
fn without_cross_read_skips_agents_md() {
    let ctx = test_context!().with_fixture("copilot/cross-read/basic");

    ctx.cast_from("copilot").arg("--force").assert().success();

    let toml = ctx.read_file("theta.toml");
    assert!(
        !toml.contains("functional style"),
        "without --cross-read, AGENTS.md content should NOT be in theta.toml:\n{toml}"
    );
    if ctx.exists(theta_static::SYSTEM_FILE_NAME) {
        let system = ctx.read_file(theta_static::SYSTEM_FILE_NAME);
        assert!(
            !system.contains("functional style"),
            "without --cross-read, AGENTS.md should NOT be in system prompt:\n{system}"
        );
    }
}
