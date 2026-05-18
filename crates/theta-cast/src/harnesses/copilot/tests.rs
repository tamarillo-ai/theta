use super::*;
use crate::ImportOptions;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use theta_harness::HarnessTarget;
use theta_harness::layout::{CopilotLayout, HarnessLayout};
use theta_schema::{
    ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, Skill, SourceRef, Subagent, Tool,
    minimal_manifest,
};

#[test]
fn produces_copilot_instructions_and_per_rule() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    fs_err::create_dir_all(src.join("rules")).unwrap();
    fs_err::write(src.join("rules/rust.md"), "Use Rust.").unwrap();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    let mut rules = BTreeMap::new();
    rules.insert(
        "rust".into(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/rust.md")),
            description: Some("Rust style".into()),
            summary: Some("this is a summary".into()),
            apply: ApplyMode::Glob,
            apply_to: Some(vec!["**/*.rs".into()]),
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = Copilot.cast_files(&m, src).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0].0,
        PathBuf::from(".github/instructions/rust.instructions.md")
    );
}

#[test]
fn glob_rule_gets_apply_to_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    fs_err::create_dir_all(src.join("rules")).unwrap();
    fs_err::write(src.join("rules/r.md"), "content").unwrap();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    let mut rules = BTreeMap::new();
    rules.insert(
        "r".into(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/r.md")),
            summary: Some("this is a summary".into()),
            description: Some("desc".into()),
            apply: ApplyMode::Glob,
            apply_to: Some(vec!["**/*.ts".into(), "**/*.tsx".into()]),
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = Copilot.cast_files(&m, src).unwrap();
    let rule_content = &files[0].1;
    assert!(rule_content.contains("applyTo:"));
    assert!(rule_content.contains("**/*.ts, **/*.tsx"));
}

#[test]
fn no_harness_config_produces_no_settings_json() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let manifest = minimal_manifest("test");
    let files = Copilot.cast_files(&manifest, src).unwrap();

    assert!(
        !files
            .iter()
            .any(|(p, _)| p == Path::new(".vscode/settings.json")),
    );
}

#[test]
fn harness_config_produces_vscode_settings() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::json!({
            "chat.agent.maxRequests": 50,
            "chat.tools.terminal": ["git *", "npm test"]
        }),
    );
    manifest.harness = Some(harness);

    let files = Copilot.cast_files(&manifest, src).unwrap();

    let settings = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/settings.json"));
    assert!(settings.is_some());

    let json: serde_json::Value = serde_json::from_str(&settings.unwrap().1).unwrap();
    assert_eq!(json["chat.agent.maxRequests"], 50);
    assert_eq!(json["chat.tools.terminal"][0], "git *");
}

#[test]
fn copilot_passthrough_fields_in_settings() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::json!({
            "chat.agent.maxRequests": 25,
            "github.copilot.chat.someCustomSetting": true
        }),
    );
    manifest.harness = Some(harness);

    let files = Copilot.cast_files(&manifest, src).unwrap();

    let settings = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/settings.json"))
        .unwrap();

    let json: serde_json::Value = serde_json::from_str(&settings.1).unwrap();
    assert_eq!(json["chat.agent.maxRequests"], 25);
    assert_eq!(json["github.copilot.chat.someCustomSetting"], true);
}

#[test]
fn tools_produce_vscode_mcp_json_with_servers_key() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();
    let mut tools = BTreeMap::new();
    tools.insert(
        "playwright".to_string(),
        Tool {
            command: Some(vec!["npx".into()]),
            args: Some(vec!["-y".into(), "@microsoft/mcp-server-playwright".into()]),
            url: None,
            env: None,
            headers: None,
            enabled: true,
        },
    );
    tools.insert(
        "github".to_string(),
        Tool {
            command: None,
            args: None,
            url: Some("https://api.githubcopilot.com/mcp".into()),
            env: None,
            headers: None,
            enabled: true,
        },
    );
    manifest.tools = Some(tools);

    let files = Copilot.cast_files(&manifest, src).unwrap();

    let mcp = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/mcp.json"))
        .expect("should produce .vscode/mcp.json");

    let json: serde_json::Value = serde_json::from_str(&mcp.1).unwrap();
    assert!(json.get("servers").is_some(), "must use 'servers' key");
    assert!(
        json.get("mcpServers").is_none(),
        "must NOT use 'mcpServers' key"
    );

    let playwright = &json["servers"]["playwright"];
    assert_eq!(playwright["type"], "stdio");
    assert_eq!(playwright["command"], "npx");
    assert_eq!(playwright["args"][0], "-y");

    let github = &json["servers"]["github"];
    assert_eq!(github["type"], "http");
    assert_eq!(github["url"], "https://api.githubcopilot.com/mcp");
}

#[test]
fn skills_are_cast_to_github_skills_dir() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    // SKILL.md must exist in .theta/skills/<name>/
    fs_err::create_dir_all(src.join("skills/testing")).unwrap();
    fs_err::write(
        src.join("skills/testing/SKILL.md"),
        "---\nname: testing\ndescription: Testing skill\n---\n\nRun tests.",
    )
    .unwrap();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();
    let mut skills = BTreeMap::new();
    skills.insert(
        "testing".to_string(),
        Skill {
            source: SourceRef::Path {
                path: "skills/testing/".into(),
            },
            tags: None,
            goal: None,
        },
    );
    manifest.skills = Some(skills);

    let files = Copilot.cast_files(&manifest, src).unwrap();

    let skill_file = files
        .iter()
        .find(|(p, _)| p == Path::new(".github/skills/testing/SKILL.md"))
        .expect("should produce .github/skills/testing/SKILL.md");

    assert!(skill_file.1.contains("Run tests."));
}

#[test]
fn subagents_are_cast_to_github_agents_dir() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();
    manifest.subagents = Some(vec![Subagent {
        name: "planner".into(),
        description: Some("Plans the work".into()),
        agent_ref: None,
        prompt_path: None,
        model: Some("claude-opus-4-5".into()),
        tools: Some(vec!["search".into(), "web".into()]),
        skills: None,
    }]);

    let files = Copilot.cast_files(&manifest, src).unwrap();

    let agent_file = files
        .iter()
        .find(|(p, _)| p == Path::new(".github/agents/planner.agent.md"))
        .expect("should produce .github/agents/planner.agent.md");

    assert!(agent_file.1.contains("description:"));
    assert!(agent_file.1.contains("Plans the work"));
    assert!(agent_file.1.contains("model:"));
    assert!(agent_file.1.contains("claude-opus-4-5"));
}

#[test]
fn import_mcp_json_servers_goes_to_tools() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path();

    fs_err::create_dir_all(project.join(".vscode")).unwrap();
    fs_err::write(
        project.join(".vscode/mcp.json"),
        serde_json::to_string(&serde_json::json!({
            "servers": {
                "playwright": {
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "@microsoft/mcp-server-playwright"]
                },
                "github": {
                    "type": "http",
                    "url": "https://api.githubcopilot.com/mcp"
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();

    let result = Copilot
        .import(project, &ImportOptions::default_for(project))
        .unwrap();
    let doc = result.document;

    // must be in [tools], NOT in [harness]
    assert!(
        doc.get("tools").is_some(),
        "tools section must exist in imported document"
    );
    assert!(
        doc.get("harness")
            .and_then(|h| h.as_table())
            .and_then(|t| t.get(HarnessTarget::Copilot.toml_key()))
            .and_then(|gp| gp.as_table())
            .and_then(|t| t.get("mcp"))
            .is_none(),
        "mcp must NOT be in [harness.github_copilot]"
    );

    let tools = doc["tools"].as_table().unwrap();
    assert!(tools.contains_key("playwright"));
    assert!(tools.contains_key("github"));

    // playwright should have command
    let playwright = tools["playwright"].as_table().unwrap();
    assert!(playwright.contains_key("command"));

    // github should have url
    let github = tools["github"].as_table().unwrap();
    assert!(github.contains_key("url"));
}

#[test]
fn import_mcp_extras_go_to_harness_github_copilot_tool() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path();

    fs_err::create_dir_all(project.join(".vscode")).unwrap();
    fs_err::write(
        project.join(".vscode/mcp.json"),
        serde_json::to_string(&serde_json::json!({
            "servers": {
                "playwright": {
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "@microsoft/mcp-server-playwright"],
                    "sandboxEnabled": true,
                    "envFile": ".env.local",
                    "dev": { "watch": "src/**/*.ts" }
                }
            },
            "inputs": [
                { "id": "gh_token", "type": "promptString", "description": "GitHub token", "password": true }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    let result = Copilot
        .import(project, &ImportOptions::default_for(project))
        .unwrap();
    let toml_str = result.document.to_string();
    let parsed: toml::Value = toml::from_str(&toml_str).unwrap();

    // theta-typed keys land in [tools.playwright]
    let playwright = parsed
        .get("tools")
        .and_then(|v| v.get("playwright"))
        .expect("playwright must be in [tools]");
    assert!(playwright.get("command").is_some());
    assert!(playwright.get("sandboxEnabled").is_none());

    // extras land in [harness.github_copilot.tool.playwright]
    let extras = parsed
        .get("harness")
        .and_then(|v| v.get(HarnessTarget::Copilot.toml_key()))
        .and_then(|v| v.get("tool"))
        .and_then(|v| v.get("playwright"))
        .expect("extras must be under [harness.github_copilot.tool.playwright]");
    assert_eq!(
        extras.get("sandboxEnabled"),
        Some(&toml::Value::Boolean(true))
    );
    assert_eq!(
        extras.get("envFile"),
        Some(&toml::Value::String(".env.local".into()))
    );
    assert!(extras.get("dev").is_some());

    // inputs array lands in [harness.github_copilot.mcp_input_variables]
    let inputs = parsed
        .get("harness")
        .and_then(|v| v.get(HarnessTarget::Copilot.toml_key()))
        .and_then(|v| v.get("mcp_input_variables"))
        .expect("inputs must be at [harness.github_copilot.mcp_input_variables]");
    let arr = inputs.as_array().expect("inputs must be an array");
    assert_eq!(arr.len(), 1);
}

#[test]
fn import_hooks_merges_into_harness_github_copilot() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path();

    fs_err::create_dir_all(project.join(".github/hooks")).unwrap();
    fs_err::write(
        project.join(".github/hooks/a.json"),
        serde_json::to_string(&serde_json::json!({
            "hooks": {
                "PreToolUse": [{ "command": "./pre.sh" }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
    fs_err::write(
        project.join(".github/hooks/b.json"),
        // bare form (no "hooks" wrapper)
        serde_json::to_string(&serde_json::json!({
            "PreToolUse": [{ "command": "./also.sh" }],
            "Stop": [{ "command": "./stop.sh" }]
        }))
        .unwrap(),
    )
    .unwrap();

    let result = Copilot
        .import(project, &ImportOptions::default_for(project))
        .unwrap();
    let doc = result.document;

    // re-serialize and round-trip as toml --> json for easy assertions
    let toml_str = doc.to_string();
    let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
    let hooks = parsed
        .get("harness")
        .and_then(|v| v.get(HarnessTarget::Copilot.toml_key()))
        .and_then(|v| v.get("hooks"))
        .expect("hooks must be under [harness.github_copilot]");

    let pre = hooks
        .get("PreToolUse")
        .and_then(|v| v.as_array())
        .expect("PreToolUse must be an array");
    assert_eq!(pre.len(), 2);

    assert!(
        hooks.get("Stop").is_some(),
        "Stop event must be present from b.json"
    );
}

#[test]
fn cast_writes_hooks_file_from_harness_config() {
    let dir = tempfile::tempdir().unwrap();
    let theta_dir = dir.path();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    // inject [harness.github_copilot.hooks] via raw TOML
    let hooks_json = serde_json::json!({
        "PreToolUse": [{ "command": "./pre.sh" }]
    });
    let mut github_copilot = serde_json::Map::new();
    github_copilot.insert("hooks".into(), hooks_json);
    let mut harness_map = std::collections::BTreeMap::new();
    harness_map.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::Value::Object(github_copilot),
    );
    m.harness = Some(harness_map);

    let files = Copilot.cast_files(&m, theta_dir).unwrap();
    let hooks_file = files
        .iter()
        .find(|(p, _)| p == &CopilotLayout::hooks_file())
        .expect("theta-hooks.json must be produced");
    let parsed: serde_json::Value = serde_json::from_str(&hooks_file.1).unwrap();
    // VS Code expects a `"hooks"` wrapper around the event map.
    let hooks = parsed
        .get("hooks")
        .expect("theta-hooks.json must wrap events under `hooks`");
    assert!(hooks.get("PreToolUse").is_some());
}

#[test]
fn cast_merges_mcp_extras_from_harness_github_copilot_tool() {
    let dir = tempfile::tempdir().unwrap();
    let theta_dir = dir.path();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();

    // theta-typed: github server with url
    let mut tools = BTreeMap::new();
    tools.insert(
        "github".to_string(),
        Tool {
            command: None,
            args: None,
            url: Some("https://api.githubcopilot.com/mcp".into()),
            env: None,
            headers: None,
            enabled: true,
        },
    );
    m.tools = Some(tools);

    // extras: [harness.github_copilot.tool.github] with sandboxEnabled + dev
    let extras = serde_json::json!({
        "github": {
            "sandboxEnabled": true,
            "dev": { "watch": "./build" }
        }
    });
    let mut gh_cfg = serde_json::Map::new();
    gh_cfg.insert("tool".into(), extras);
    let mut harness_map = std::collections::BTreeMap::new();
    harness_map.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::Value::Object(gh_cfg),
    );
    m.harness = Some(harness_map);

    let files = Copilot.cast_files(&m, theta_dir).unwrap();
    let mcp = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/mcp.json"))
        .expect("must produce .vscode/mcp.json");
    let json: serde_json::Value = serde_json::from_str(&mcp.1).unwrap();

    let github = &json["servers"]["github"];
    // theta-typed fields present
    assert_eq!(github["type"], "http");
    assert_eq!(github["url"], "https://api.githubcopilot.com/mcp");
    // extras merged in
    assert_eq!(github["sandboxEnabled"], true);
    assert_eq!(github["dev"]["watch"], "./build");
}

#[test]
fn cast_emits_pure_extras_only_mcp_server() {
    let dir = tempfile::tempdir().unwrap();
    let theta_dir = dir.path();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    // no [tools] at all

    let extras = serde_json::json!({
        "custom-local": {
            "command": "./my-server",
            "sandboxEnabled": false
        }
    });
    let mut gh_cfg = serde_json::Map::new();
    gh_cfg.insert("tool".into(), extras);
    let mut harness_map = std::collections::BTreeMap::new();
    harness_map.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::Value::Object(gh_cfg),
    );
    m.harness = Some(harness_map);

    let files = Copilot.cast_files(&m, theta_dir).unwrap();
    let mcp = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/mcp.json"))
        .expect("must produce .vscode/mcp.json even with only extras");
    let json: serde_json::Value = serde_json::from_str(&mcp.1).unwrap();
    assert_eq!(json["servers"]["custom-local"]["command"], "./my-server");
    assert_eq!(json["servers"]["custom-local"]["sandboxEnabled"], false);
}

#[test]
fn cast_writes_mcp_input_variables_at_top_level() {
    let dir = tempfile::tempdir().unwrap();
    let theta_dir = dir.path();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();

    let inputs = serde_json::json!([
        { "type": "promptString", "id": "api-key", "description": "API key", "password": true }
    ]);
    let mut gh_cfg = serde_json::Map::new();
    gh_cfg.insert("mcp_input_variables".into(), inputs);
    let mut harness_map = std::collections::BTreeMap::new();
    harness_map.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::Value::Object(gh_cfg),
    );
    m.harness = Some(harness_map);

    let files = Copilot.cast_files(&m, theta_dir).unwrap();
    let mcp = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/mcp.json"))
        .expect("must produce .vscode/mcp.json when inputs are present");
    let json: serde_json::Value = serde_json::from_str(&mcp.1).unwrap();
    let inputs = json["inputs"].as_array().unwrap();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0]["id"], "api-key");
    assert_eq!(inputs[0]["type"], "promptString");
}

#[test]
fn import_skills_from_github_skills_dir() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path();

    fs_err::create_dir_all(project.join(".github/skills/web-testing")).unwrap();
    fs_err::write(
        project.join(".github/skills/web-testing/SKILL.md"),
        "---\nname: web-testing\ndescription: Tests web pages\n---\n\nRun Playwright.",
    )
    .unwrap();

    let result = Copilot
        .import(project, &ImportOptions::default_for(project))
        .unwrap();
    let doc = result.document;

    assert!(doc.get("skills").is_some(), "skills section must exist");
    let skills = doc["skills"].as_table().unwrap();
    assert!(skills.contains_key("web-testing"));

    // extracted file should contain SKILL.md content
    assert!(
        result
            .extracted_files
            .iter()
            .any(|(p, _)| p == Path::new("skills/web-testing/SKILL.md"))
    );
}

#[test]
fn import_agents_from_github_agents_dir() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path();

    fs_err::create_dir_all(project.join(".github/agents")).unwrap();
    fs_err::write(
        project.join(".github/agents/planner.agent.md"),
        "---\ndescription: Plans the work\nmodel: claude-opus-4-5\ntools:\n  - search\n  - web\n---\n\nYou are a planning agent.",
    )
    .unwrap();

    let result = Copilot
        .import(project, &ImportOptions::default_for(project))
        .unwrap();
    let doc = result.document;

    let subagents = doc.get("subagents").expect("subagents must exist");
    let arr = subagents.as_array_of_tables().unwrap();
    assert_eq!(arr.len(), 1);

    let entry = arr.iter().next().unwrap();
    assert_eq!(entry["name"].as_str().unwrap(), "planner");
    assert_eq!(entry["description"].as_str().unwrap(), "Plans the work");
    assert_eq!(entry["model"].as_str().unwrap(), "claude-opus-4-5");
    assert!(entry.get("prompt").is_none(), "prompt key should not exist");
}

#[test]
fn import_subagent_extras_go_to_harness_github_copilot_subagent() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path();

    fs_err::create_dir_all(project.join(".github/agents")).unwrap();
    // planner has typed fields + unmodeled extras
    fs_err::write(
        project.join(".github/agents/planner.agent.md"),
        "---\ndescription: Plans\nmodel: claude-opus-4-5\ntools:\n  - search\nargument-hint: \"Query\"\nuser-invocable: true\nhandoffs:\n  - executor\ntarget: vscode\n---\n\nBody",
    )
    .unwrap();

    let result = Copilot
        .import(project, &ImportOptions::default_for(project))
        .unwrap();

    // round-trip via string so we can navigate with toml::Value
    let s = result.document.to_string();
    let parsed: toml::Value = toml::from_str(&s).unwrap();

    let gp = &parsed["harness"][HarnessTarget::Copilot.toml_key()];
    let sub = &gp["subagent"]["planner"];
    assert_eq!(sub["argument-hint"].as_str().unwrap(), "Query");
    assert!(sub["user-invocable"].as_bool().unwrap());
    assert_eq!(sub["target"].as_str().unwrap(), "vscode");
    let handoffs = sub["handoffs"].as_array().unwrap();
    assert_eq!(handoffs.len(), 1);
    assert_eq!(handoffs[0].as_str().unwrap(), "executor");

    // typed keys must NOT appear in extras
    assert!(sub.as_table().unwrap().get("description").is_none());
    assert!(sub.as_table().unwrap().get("model").is_none());
    assert!(sub.as_table().unwrap().get("tools").is_none());
    assert!(sub.as_table().unwrap().get("name").is_none());
}

#[test]
fn cast_merges_subagent_extras_into_agent_md_frontmatter() {
    use theta_schema::Subagent;

    let dir = tempfile::tempdir().unwrap();
    let theta_dir = dir.path();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    m.subagents = Some(vec![Subagent {
        name: "planner".into(),
        description: Some("Plans".into()),
        model: Some("claude-opus-4-5".into()),
        tools: Some(vec!["search".into()]),
        prompt_path: None,
        agent_ref: None,
        skills: None,
    }]);

    let extras = serde_json::json!({
        "planner": {
            "argument-hint": "Query",
            "user-invocable": true,
            "handoffs": ["executor"]
        }
    });
    let mut gh_cfg = serde_json::Map::new();
    gh_cfg.insert("subagent".into(), extras);
    let mut harness_map = std::collections::BTreeMap::new();
    harness_map.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::Value::Object(gh_cfg),
    );
    m.harness = Some(harness_map);

    let files = Copilot.cast_files(&m, theta_dir).unwrap();
    let agent_file = files
        .iter()
        .find(|(p, _)| p == &CopilotLayout::agent("planner"))
        .expect("planner.agent.md must be produced");

    let content = &agent_file.1;
    // typed fields present
    assert!(content.contains("description: Plans"));
    assert!(content.contains("model: claude-opus-4-5"));
    // extras merged into frontmatter
    assert!(
        content.contains("argument-hint: Query"),
        "argument-hint missing in:\n{content:?}"
    );
    assert!(
        content.contains("user-invocable: true"),
        "user-invocable missing in:\n{content:?}"
    );
    assert!(
        content.contains("handoffs:"),
        "handoffs missing in:\n{content:?}"
    );
    assert!(content.contains("- executor"));
}

#[test]
fn cast_skips_extras_that_conflict_with_typed_fields() {
    use theta_schema::Subagent;

    let dir = tempfile::tempdir().unwrap();
    let theta_dir = dir.path();

    let mut m = minimal_manifest("test");
    m.agent.description = "Test".into();
    m.subagents = Some(vec![Subagent {
        name: "planner".into(),
        description: Some("Typed desc".into()),
        model: Some("claude-opus-4-5".into()),
        tools: None,
        prompt_path: None,
        agent_ref: None,
        skills: None,
    }]);

    // extras try to override `description` AND `model` AND `name`
    let extras = serde_json::json!({
        "planner": {
            "description": "Extras desc (should lose)",
            "model": "gpt-5",
            "name": "should-not-appear",
            "argument-hint": "allowed"
        }
    });
    let mut gh_cfg = serde_json::Map::new();
    gh_cfg.insert("subagent".into(), extras);
    let mut harness_map = std::collections::BTreeMap::new();
    harness_map.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::Value::Object(gh_cfg),
    );
    m.harness = Some(harness_map);

    let files = Copilot.cast_files(&m, theta_dir).unwrap();
    let agent_file = files
        .iter()
        .find(|(p, _)| p == &CopilotLayout::agent("planner"))
        .expect("planner.agent.md must be produced");

    let content = &agent_file.1;
    // theta-typed wins
    assert!(content.contains("description: Typed desc"));
    assert!(content.contains("model: claude-opus-4-5"));
    assert!(!content.contains("Extras desc"));
    assert!(!content.contains("gpt-5"));
    assert!(!content.contains("should-not-appear"));
    // non-conflicting extras still appear
    assert!(content.contains("argument-hint: allowed"));
}

/// `settings.json`: unknown user keys (e.g. `editor.fontSize`) MUST round-trip
/// through a cast when an existing file sits at `output_dir`.
#[test]
fn cast_with_output_preserves_unknown_settings_json_keys() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let theta_dir = root.join(".theta");
    fs_err::create_dir_all(&theta_dir).unwrap();

    // seed an existing .vscode/settings.json with an unrelated key
    fs_err::create_dir_all(root.join(".vscode")).unwrap();
    fs_err::write(
        root.join(".vscode/settings.json"),
        r#"{
  "editor.fontSize": 14,
  "chat.agent.maxRequests": 10
}"#,
    )
    .unwrap();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test".into();
    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::json!({ "chat.agent.maxRequests": 50 }),
    );
    manifest.harness = Some(harness);

    let files = Copilot
        .cast_files_with_output(&manifest, &theta_dir, root)
        .unwrap();

    let (_, content) = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/settings.json"))
        .expect("settings.json must be produced");
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    // non-copilot key preserved from existing file
    assert_eq!(
        parsed
            .get("editor.fontSize")
            .and_then(serde_json::Value::as_i64),
        Some(14)
    );
    // theta-owned key overwritten (50, not the old 10)
    assert_eq!(
        parsed
            .get("chat.agent.maxRequests")
            .and_then(serde_json::Value::as_i64),
        Some(50)
    );
}

/// `mcp.json`: servers NOT owned by theta (no entry in `[tools]`/`[harness.*.tool]`)
/// MUST round-trip through a cast when an existing file sits at `output_dir`
#[test]
fn cast_with_output_preserves_unknown_mcp_servers() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let theta_dir = root.join(".theta");
    fs_err::create_dir_all(&theta_dir).unwrap();

    fs_err::create_dir_all(root.join(".vscode")).unwrap();
    fs_err::write(
        root.join(".vscode/mcp.json"),
        r#"{
  "servers": {
    "unrelated": { "type": "stdio", "command": "/usr/bin/unrelated" },
    "theta-managed": { "type": "stdio", "command": "OLD" }
  }
}"#,
    )
    .unwrap();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test".into();
    let mut tools = BTreeMap::new();
    tools.insert(
        "theta-managed".into(),
        Tool {
            enabled: true,
            command: Some(vec!["NEW".into()]),
            url: None,
            env: None,
            headers: None,
            args: None,
        },
    );
    manifest.tools = Some(tools);

    let files = Copilot
        .cast_files_with_output(&manifest, &theta_dir, root)
        .unwrap();

    let (_, content) = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/mcp.json"))
        .expect("mcp.json must be produced");
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    let servers = parsed.get("servers").and_then(|v| v.as_object()).unwrap();
    // unrelated server preserved verbatim
    assert_eq!(
        servers
            .get("unrelated")
            .and_then(|v| v.get("command"))
            .and_then(|v| v.as_str()),
        Some("/usr/bin/unrelated")
    );
    // theta-owned server overwritten (NEW, not OLD)
    assert_eq!(
        servers
            .get("theta-managed")
            .and_then(|v| v.get("command"))
            .and_then(|v| v.as_str()),
        Some("NEW")
    );
}

/// `mcp.json`: when theta produces no `inputs` but the existing file has them,
/// the existing `inputs` MUST round-trip; unrelated top-level keys MUST also
/// round-trip.
#[test]
fn cast_with_output_preserves_existing_mcp_inputs_and_unknown_top_level_keys() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let theta_dir = root.join(".theta");
    fs_err::create_dir_all(&theta_dir).unwrap();

    fs_err::create_dir_all(root.join(".vscode")).unwrap();
    fs_err::write(
        root.join(".vscode/mcp.json"),
        r#"{
  "servers": { "unrelated": { "type": "stdio", "command": "x" } },
  "inputs": [ { "id": "api_key", "type": "promptString" } ],
  "metadata": { "owner": "user" }
}"#,
    )
    .unwrap();

    // manifest contributes a server but NO mcp_input_variables
    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test".into();
    let mut tools = BTreeMap::new();
    tools.insert(
        "theta-managed".into(),
        Tool {
            enabled: true,
            command: Some(vec!["managed".into()]),
            url: None,
            env: None,
            headers: None,
            args: None,
        },
    );
    manifest.tools = Some(tools);

    let files = Copilot
        .cast_files_with_output(&manifest, &theta_dir, root)
        .unwrap();

    let (_, content) = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/mcp.json"))
        .expect("mcp.json must be produced");
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    // existing inputs preserved (theta produced none)
    let inputs = parsed.get("inputs").and_then(|v| v.as_array()).unwrap();
    assert_eq!(inputs.len(), 1);
    assert_eq!(
        inputs[0].get("id").and_then(|v| v.as_str()),
        Some("api_key")
    );
    // unknown top-level key preserved
    assert_eq!(
        parsed
            .get("metadata")
            .and_then(|v| v.get("owner"))
            .and_then(|v| v.as_str()),
        Some("user")
    );
}

/// the default trait path (`cast_files`) MUST behave as a from-scratch cast
/// - no on-disk reads, no merge - even when `output_dir` has content
#[test]
fn cast_files_without_output_dir_is_from_scratch() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let theta_dir = root.join(".theta");
    fs_err::create_dir_all(&theta_dir).unwrap();

    // seed an existing file that SHOULD be ignored by `cast_files`
    fs_err::create_dir_all(root.join(".vscode")).unwrap();
    fs_err::write(
        root.join(".vscode/settings.json"),
        r#"{ "editor.fontSize": 14 }"#,
    )
    .unwrap();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test".into();
    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::Copilot.toml_key().to_string(),
        serde_json::json!({ "chat.agent.maxRequests": 50 }),
    );
    manifest.harness = Some(harness);

    let files = Copilot.cast_files(&manifest, &theta_dir).unwrap();
    let (_, content) = files
        .iter()
        .find(|(p, _)| p == Path::new(".vscode/settings.json"))
        .expect("settings.json must be produced");
    let parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    // only theta-owned key present; the seeded `editor.fontSize` is NOT
    // read because `cast_files` is pure / from-scratch
    assert_eq!(
        parsed
            .get("chat.agent.maxRequests")
            .and_then(serde_json::Value::as_i64),
        Some(50)
    );
    assert!(parsed.get("editor.fontSize").is_none());
}
