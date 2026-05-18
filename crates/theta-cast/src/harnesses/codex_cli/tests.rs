use super::*;
use crate::{Caster, ImportOptions, Importer};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use theta_harness::HarnessTarget;
use theta_schema::{
    ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, Tool, minimal_manifest,
};

#[test]
fn produces_single_agents_md() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    fs_err::create_dir_all(src.join("rules")).unwrap();
    fs_err::write(src.join("rules/safety.md"), "Be safe.").unwrap();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    let mut rules = BTreeMap::new();
    rules.insert(
        "safety".into(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/safety.md")),
            summary: Some("this is a summary".into()),
            description: None,
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = CodexCli.cast_files(&m, src).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, PathBuf::from("AGENTS.md"));
    // codex AGENTS.md is opaque body per decision D4 - no synthetic `# name`
    // header is prepended. only the rule body and the `## <name>` section
    // heading for rules appear.
    assert!(!files[0].1.contains("# test"));
    assert!(files[0].1.contains("## safety"));
    assert!(files[0].1.contains("Be safe."));
}

#[test]
fn no_per_rule_files() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    fs_err::create_dir_all(src.join("rules")).unwrap();
    fs_err::write(src.join("rules/a.md"), "A").unwrap();
    fs_err::write(src.join("rules/b.md"), "B").unwrap();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    let mut rules = BTreeMap::new();
    rules.insert(
        "a".into(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/a.md")),
            summary: Some("this is a summary".into()),
            description: None,
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    rules.insert(
        "b".into(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/b.md")),
            summary: Some("this is a summary".into()),
            description: None,
            apply: ApplyMode::Glob,
            apply_to: Some(vec!["*.rs".into()]),
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = CodexCli.cast_files(&m, src).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].1.contains('A'));
    assert!(files[0].1.contains('B'));
}

#[test]
fn no_harness_config_produces_no_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let manifest = minimal_manifest("test");
    let files = CodexCli.cast_files(&manifest, src).unwrap();

    assert!(
        !files
            .iter()
            .any(|(p, _)| p == Path::new(".codex/config.toml")),
    );
}

#[test]
fn harness_config_produces_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::CodexCli.toml_key().to_string(),
        serde_json::json!({
            "sandbox_mode": "workspace-write",
            "approval_policy": "on-request",
            "web_search": "cached",
            "personality": "pragmatic"
        }),
    );
    manifest.harness = Some(harness);

    let files = CodexCli.cast_files(&manifest, src).unwrap();

    let config = files
        .iter()
        .find(|(p, _)| p == Path::new(".codex/config.toml"));
    assert!(config.is_some());

    let content = &config.unwrap().1;
    assert!(content.contains("sandbox_mode = \"workspace-write\""));
    assert!(content.contains("approval_policy = \"on-request\""));
    assert!(content.contains("web_search = \"cached\""));
    assert!(content.contains("personality = \"pragmatic\""));
}

#[test]
fn codex_features_table_in_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::CodexCli.toml_key().to_string(),
        serde_json::json!({
            "sandbox_mode": "workspace-write",
            "features": {
                "multi_agent": true,
                "undo": true
            }
        }),
    );
    manifest.harness = Some(harness);

    let files = CodexCli.cast_files(&manifest, src).unwrap();

    let config = files
        .iter()
        .find(|(p, _)| p == Path::new(".codex/config.toml"))
        .unwrap();

    let content = &config.1;
    assert!(content.contains("[features]"));
    assert!(content.contains("multi_agent = true"));
    assert!(content.contains("undo = true"));
}

#[test]
fn codex_passthrough_fields_in_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::CodexCli.toml_key().to_string(),
        serde_json::json!({
            "sandbox_mode": "workspace-write",
            "shell_environment_policy": "inherit"
        }),
    );
    manifest.harness = Some(harness);

    let files = CodexCli.cast_files(&manifest, src).unwrap();

    let config = files
        .iter()
        .find(|(p, _)| p == Path::new(".codex/config.toml"))
        .unwrap();

    let content = &config.1;
    assert!(content.contains("sandbox_mode = \"workspace-write\""));
    assert!(content.contains("shell_environment_policy = \"inherit\""));
}

#[test]
fn cast_disabled_tool_emits_enabled_false() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();
    manifest.tools = Some(BTreeMap::from([(
        "playwright".to_string(),
        Tool {
            command: Some(vec!["npx".into(), "@playwright/mcp".into()]),
            url: None,
            env: None,
            headers: None,
            args: None,
            enabled: false,
        },
    )]));

    let files = CodexCli.cast_files(&manifest, src).unwrap();
    let config = files
        .iter()
        .find(|(p, _)| p == Path::new(".codex/config.toml"))
        .unwrap();

    assert!(config.1.contains("[mcp_servers.playwright]"));
    assert!(config.1.contains("enabled = false"));
}

#[test]
fn import_mcp_servers_go_to_tools() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // create .codex/config.toml with mcp_servers + other settings
    fs_err::create_dir_all(root.join(".codex")).unwrap();
    fs_err::write(
        root.join(".codex/config.toml"),
        r#"
sandbox_mode = "workspace-write"

[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]

[mcp_servers.context7.env]
MY_VAR = "val"

[mcp_servers.figma]
url = "https://mcp.figma.com/mcp"
"#,
    )
    .unwrap();

    let result = CodexCli
        .import(root, &ImportOptions::default_for(root))
        .unwrap();
    let toml = result.document.to_string();

    // mcp_servers should become [tools]
    assert!(toml.contains("[tools.context7]"), "tools.context7 missing");
    assert!(
        toml.contains("command = [\"npx\"]"),
        "command array missing"
    );
    assert!(toml.contains("@upstash/context7-mcp"), "args missing");
    assert!(toml.contains("[tools.figma]"), "tools.figma missing");
    assert!(
        toml.contains("url = \"https://mcp.figma.com/mcp\""),
        "figma url missing"
    );

    // sandbox_mode should be in [harness.codex], NOT in tools
    assert!(toml.contains("[harness.codex]"), "harness.codex missing");
    assert!(
        toml.contains("sandbox_mode = \"workspace-write\""),
        "sandbox_mode missing in harness"
    );

    // mcp_servers should NOT appear in harness.codex
    assert!(
        !toml.contains("[harness.codex.mcp_servers"),
        "mcp_servers leaked to harness.codex"
    );
}

#[test]
fn import_disabled_mcp_server_preserves_enabled_false() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs_err::create_dir_all(root.join(".codex")).unwrap();
    fs_err::write(
        root.join(".codex/config.toml"),
        r#"
[mcp_servers.context7]
command = "npx"
args = ["-y", "@upstash/context7-mcp"]
enabled = false
"#,
    )
    .unwrap();

    let result = CodexCli
        .import(root, &ImportOptions::default_for(root))
        .unwrap();
    let toml = result.document.to_string();

    assert!(toml.contains("[tools.context7]"), "tools.context7 missing");
    assert!(toml.contains("enabled = false"), "enabled flag missing");
}

#[test]
fn import_subagents_from_codex_agents_dir() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs_err::create_dir_all(root.join(".codex/agents")).unwrap();
    fs_err::write(
        root.join(".codex/agents/reviewer.toml"),
        r#"
name = "reviewer"
description = "PR reviewer focused on correctness."
model = "gpt-5.4"
developer_instructions = """
Review code like an owner.
Prioritize correctness and security.
"""
"#,
    )
    .unwrap();
    fs_err::write(
        root.join(".codex/agents/explorer.toml"),
        r#"
name = "explorer"
description = "Read-only codebase explorer."
sandbox_mode = "read-only"
developer_instructions = "Stay in exploration mode."
"#,
    )
    .unwrap();

    let result = CodexCli
        .import(root, &ImportOptions::default_for(root))
        .unwrap();
    let toml = result.document.to_string();

    // both subagents should be present
    assert!(toml.contains("[[subagents]]"), "subagents array missing");
    assert!(toml.contains("name = \"explorer\""), "explorer missing");
    assert!(toml.contains("name = \"reviewer\""), "reviewer missing");
    assert!(toml.contains("model = \"gpt-5.4\""), "model missing");

    assert!(
        toml.contains("[harness.codex.subagent.explorer]"),
        "expected [harness.codex.subagent.explorer] table, got:\n{toml}"
    );
    assert!(
        toml.contains("sandbox_mode = \"read-only\""),
        "expected sandbox_mode under subagent extras, got:\n{toml}"
    );
}

#[test]
fn import_mcp_unmodeled_fields_round_trip_via_harness_extras() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs_err::create_dir_all(root.join(".codex")).unwrap();
    fs_err::write(
        root.join(".codex/config.toml"),
        r#"
[mcp_servers.chrome]
url = "http://localhost:3000/mcp"
startup_timeout_sec = 20
enabled_tools = ["open", "screenshot"]
"#,
    )
    .unwrap();

    let result = CodexCli
        .import(root, &ImportOptions::default_for(root))
        .unwrap();
    let toml = result.document.to_string();

    // theta-typed MCP keys land in [tools.<name>]
    assert!(toml.contains("[tools.chrome]"), "tools.chrome missing");
    assert!(
        toml.contains("url = \"http://localhost:3000/mcp\""),
        "url missing"
    );

    // codex-specific MCP fields not in theta-typed set MUST round-trip via
    // [harness.codex.tool.<name>] extras (decision G3). they used to be
    // dropped + hinted, but the per-server extras passthrough now preserves
    // them so cast can reconstruct the original config.toml byte-faithfully.
    assert!(
        toml.contains("[harness.codex.tool.chrome]"),
        "expected [harness.codex.tool.chrome] for MCP extras, got:\n{toml}"
    );
    assert!(
        toml.contains("startup_timeout_sec"),
        "expected startup_timeout_sec under tool extras, got:\n{toml}"
    );
    assert!(
        toml.contains("enabled_tools"),
        "expected enabled_tools under tool extras, got:\n{toml}"
    );
    let _ = result;
}

#[test]
fn import_rules_dir_emits_hint() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs_err::create_dir_all(root.join(".codex/rules")).unwrap();
    fs_err::write(
        root.join(".codex/rules/default.rules"),
        "prefix_rule(pattern = [\"rm\"], decision = \"forbidden\")",
    )
    .unwrap();

    let result = CodexCli
        .import(root, &ImportOptions::default_for(root))
        .unwrap();

    let hint = result
        .diagnostics
        .iter()
        .find(|d| d.message.contains("exec-policy"));
    assert!(hint.is_some(), "expected hint for .codex/rules/");
}
