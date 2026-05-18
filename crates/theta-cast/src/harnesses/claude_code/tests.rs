use super::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use theta_harness::HarnessTarget;
use theta_schema::{
    ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, ThetaManifest, minimal_manifest,
};

use crate::Caster;

fn manifest_with_rule(apply: ApplyMode, apply_to: Option<Vec<String>>) -> ThetaManifest {
    let mut m = minimal_manifest("test");
    m.agent.description = "Test agent".into();
    let mut rules = BTreeMap::new();
    rules.insert(
        "safety".into(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/safety.md")),
            summary: Some("this is a summary".into()),
            description: Some("Safety".into()),
            apply,
            apply_to,
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });
    m
}

#[test]
fn rule_only_manifest_skips_claude_md() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    fs_err::create_dir_all(src.join("rules")).unwrap();
    fs_err::write(src.join("rules/safety.md"), "Be safe.").unwrap();

    let manifest = manifest_with_rule(ApplyMode::Always, None);
    let files = ClaudeCode.cast_files(&manifest, src).unwrap();

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, PathBuf::from(".claude/rules/safety.md"));
    assert!(files[0].1.contains("Be safe."));
    assert!(
        !files.iter().any(|(p, _)| p == Path::new("CLAUDE.md")),
        "CLAUDE.md must not be emitted when [instructions].system is unset"
    );
}

#[test]
fn glob_rule_gets_paths_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    fs_err::create_dir_all(src.join("rules")).unwrap();
    fs_err::write(src.join("rules/safety.md"), "content").unwrap();

    let manifest = manifest_with_rule(ApplyMode::Glob, Some(vec!["**/*.rs".into()]));
    let files = ClaudeCode.cast_files(&manifest, src).unwrap();

    let rule_content = &files[0].1;
    assert!(rule_content.contains("paths:"));
    assert!(rule_content.contains("**/*.rs"));
}

#[test]
fn always_rule_has_no_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();
    fs_err::create_dir_all(src.join("rules")).unwrap();
    fs_err::write(src.join("rules/safety.md"), "content").unwrap();

    let manifest = manifest_with_rule(ApplyMode::Always, None);
    let files = ClaudeCode.cast_files(&manifest, src).unwrap();

    assert!(!files[0].1.contains("---"));
}

#[test]
fn no_harness_config_produces_no_settings_json() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let manifest = minimal_manifest("test");
    let files = ClaudeCode.cast_files(&manifest, src).unwrap();

    assert!(
        !files
            .iter()
            .any(|(p, _)| p == Path::new(".claude/settings.json")),
        "settings.json should not be produced without harness config"
    );
}

#[test]
fn harness_config_produces_settings_json() {
    // theta.toml stores claude settings with their native camelCase keys.
    // cast emits them verbatim into .claude/settings.json with no remapping.
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::ClaudeCode.toml_key().to_string(),
        serde_json::json!({
            "sandbox": { "enabled": true },
            "viewMode": "focus"
        }),
    );
    manifest.harness = Some(harness);

    let files = ClaudeCode.cast_files(&manifest, src).unwrap();

    let settings_file = files
        .iter()
        .find(|(p, _)| p == Path::new(".claude/settings.json"))
        .expect("expected settings.json in cast output");

    let json: serde_json::Value = serde_json::from_str(&settings_file.1).unwrap();
    assert_eq!(json["sandbox"]["enabled"], true);
    assert_eq!(json["viewMode"], "focus");
}

#[test]
fn passthrough_fields_appear_in_settings_json() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::ClaudeCode.toml_key().to_string(),
        serde_json::json!({
            "sandbox": { "enabled": false },
            "contextWindow": { "maxTokens": 200_000 },
            "customUnknownField": "preserved"
        }),
    );
    manifest.harness = Some(harness);

    let files = ClaudeCode.cast_files(&manifest, src).unwrap();

    let settings_file = files
        .iter()
        .find(|(p, _)| p == Path::new(".claude/settings.json"))
        .unwrap();

    let json: serde_json::Value = serde_json::from_str(&settings_file.1).unwrap();
    assert_eq!(json["sandbox"]["enabled"], false);
    // passthrough fields preserved
    assert_eq!(json["contextWindow"]["maxTokens"], 200_000);
    assert_eq!(json["customUnknownField"], "preserved");
}

#[test]
fn enabled_plugins_preserves_camel_case() {
    // claude-spec keys use camelCase natively (enabledPlugins, autoMode, viewMode, ...).
    // theta.toml stores them as-is; cast emits them verbatim. no remapping.
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::ClaudeCode.toml_key().to_string(),
        serde_json::json!({
            "enabledPlugins": { "some-plugin@marketplace": true }
        }),
    );
    manifest.harness = Some(harness);

    let files = ClaudeCode.cast_files(&manifest, src).unwrap();

    let settings_file = files
        .iter()
        .find(|(p, _)| p == Path::new(".claude/settings.json"))
        .unwrap();

    let json: serde_json::Value = serde_json::from_str(&settings_file.1).unwrap();
    assert_eq!(json["enabledPlugins"]["some-plugin@marketplace"], true);
}
