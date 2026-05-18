use super::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use theta_harness::HarnessTarget;
use theta_schema::{ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, minimal_manifest};

#[test]
fn produces_system_and_per_rule_mdc() {
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
            description: Some("Safety".into()),
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = CursorHarness.cast_files(&m, src).unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].0, PathBuf::from(".cursor/rules/system.md"));
    assert!(files[0].1.contains("alwaysApply: true"));
    assert_eq!(files[1].0, PathBuf::from(".cursor/rules/safety.mdc"));
    assert!(files[1].1.contains("alwaysApply: true"));
    assert!(files[1].1.contains("Be safe."));
}

#[test]
fn glob_rule_gets_globs_frontmatter() {
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
            apply_to: Some(vec!["**/*.rs".into()]),
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = CursorHarness.cast_files(&m, src).unwrap();
    let rule_content = &files[1].1;
    assert!(rule_content.contains("alwaysApply: false"));
    assert!(rule_content.contains("globs:"));
    assert!(rule_content.contains("**/*.rs"));
}

#[test]
fn model_decision_maps_to_always_apply_false() {
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
            description: Some("Decide yourself".into()),
            apply: ApplyMode::ModelDecision,
            apply_to: None,
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = CursorHarness.cast_files(&m, src).unwrap();
    let rule_content = &files[1].1;
    assert!(rule_content.contains("alwaysApply: false"));
    assert!(!rule_content.contains("globs:"));
}

#[test]
fn no_harness_config_produces_no_hooks_json() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let manifest = minimal_manifest("test");
    let files = CursorHarness.cast_files(&manifest, src).unwrap();

    assert!(
        !files
            .iter()
            .any(|(p, _)| p == Path::new(".cursor/hooks.json")),
    );
}

#[test]
fn harness_config_produces_hooks_json() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::Cursor.toml_key().to_string(),
        serde_json::json!({
            "hooks": {
                "preToolUse": [
                    { "command": "./scripts/check.sh", "matcher": "Bash", "timeout": 10 }
                ]
            }
        }),
    );
    manifest.harness = Some(harness);

    let files = CursorHarness.cast_files(&manifest, src).unwrap();

    let hooks = files
        .iter()
        .find(|(p, _)| p == Path::new(".cursor/hooks.json"));
    assert!(hooks.is_some());

    let json: serde_json::Value = serde_json::from_str(&hooks.unwrap().1).unwrap();
    assert_eq!(json["preToolUse"][0]["command"], "./scripts/check.sh");
    assert_eq!(json["preToolUse"][0]["timeout"], 10);
}

#[test]
fn cursor_without_hooks_produces_no_hooks_json() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test agent".into();

    let mut harness = BTreeMap::new();
    harness.insert(
        HarnessTarget::Cursor.toml_key().to_string(),
        serde_json::json!({
            "import_rules": ["https://github.com/company/rules"]
        }),
    );
    manifest.harness = Some(harness);

    let files = CursorHarness.cast_files(&manifest, src).unwrap();

    assert!(
        !files
            .iter()
            .any(|(p, _)| p == Path::new(".cursor/hooks.json")),
    );
}
// mdc frontmatter parser

use super::import::{parse_mdc_frontmatter, parse_mdc_globs};

#[test]
fn mdc_basic_always_apply() {
    let input = "---\nalwaysApply: true\n---\nBe safe.";
    let mdc = parse_mdc_frontmatter(input);
    assert!(mdc.always_apply);
    assert!(mdc.description.is_none());
    assert!(mdc.globs.is_none());
    assert_eq!(mdc.body, "Be safe.");
}

#[test]
fn mdc_description_with_colon() {
    let input = "---\ndescription: Go patterns: Repository, Adapter\nglobs: [\"**/*.go\"]\nalwaysApply: false\n---\nbody";
    let mdc = parse_mdc_frontmatter(input);
    assert!(!mdc.always_apply);
    assert_eq!(
        mdc.description.as_deref(),
        Some("Go patterns: Repository, Adapter")
    );
    assert_eq!(mdc.globs.as_deref(), Some(&["**/*.go".to_string()][..]));
    assert_eq!(mdc.body, "body");
}

#[test]
fn mdc_leading_star_glob() {
    let input = "---\nglobs: *.ts\nalwaysApply: false\n---\nUse strict TypeScript.";
    let mdc = parse_mdc_frontmatter(input);
    assert!(!mdc.always_apply);
    assert_eq!(mdc.globs.as_deref(), Some(&["*.ts".to_string()][..]));
    assert_eq!(mdc.body, "Use strict TypeScript.");
}

#[test]
fn mdc_comma_separated_globs() {
    let input = "---\nglobs: *.ts, *.tsx\nalwaysApply: false\n---\nbody";
    let mdc = parse_mdc_frontmatter(input);
    assert_eq!(
        mdc.globs.as_deref(),
        Some(&["*.ts".to_string(), "*.tsx".to_string()][..])
    );
}

#[test]
fn mdc_json_array_globs() {
    let input =
        "---\nglobs: [\"internal/**/*.go\", \"pkg/**/*.go\"]\nalwaysApply: false\n---\nbody";
    let mdc = parse_mdc_frontmatter(input);
    assert_eq!(
        mdc.globs.as_deref(),
        Some(&["internal/**/*.go".to_string(), "pkg/**/*.go".to_string()][..])
    );
}

#[test]
fn mdc_quoted_glob_string() {
    let input = "---\nglobs: \"docs/**/*.md, docs/**/*.mdx\"\nalwaysApply: false\n---\nbody";
    let mdc = parse_mdc_frontmatter(input);
    assert_eq!(
        mdc.globs.as_deref(),
        Some(&["docs/**/*.md".to_string(), "docs/**/*.mdx".to_string()][..])
    );
}

#[test]
fn mdc_empty_description_skipped() {
    let input = "---\ndescription: \nglobs: src/**\nalwaysApply: false\n---\nbody";
    let mdc = parse_mdc_frontmatter(input);
    assert!(mdc.description.is_none());
    assert_eq!(mdc.globs.as_deref(), Some(&["src/**".to_string()][..]));
}

#[test]
fn mdc_no_frontmatter() {
    let input = "just plain markdown\nno frontmatter";
    let mdc = parse_mdc_frontmatter(input);
    assert!(!mdc.always_apply);
    assert!(mdc.description.is_none());
    assert!(mdc.globs.is_none());
}

#[test]
fn mdc_unknown_keys_ignored() {
    let input = "---\nalwaysApply: true\ncustomKey: whatever\n---\nbody";
    let mdc = parse_mdc_frontmatter(input);
    assert!(mdc.always_apply);
    assert_eq!(mdc.body, "body");
}

#[test]
fn mdc_globs_brace_expansion_matches_cursor_behavior() {
    // brace expansion is broken in Cursor itself - commas inside braces
    // are consumed by the splitter. we match that behavior.
    let vals = parse_mdc_globs("\"**/*.{ts,tsx,js,jsx}\"");
    assert_eq!(vals, vec!["**/*.{ts", "tsx", "js", "jsx}"]);
}

#[test]
fn cast_rule_emits_comma_separated_globs() {
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
            summary: None,
            description: None,
            apply: ApplyMode::Glob,
            apply_to: Some(vec!["src/**/*.ts".into(), "lib/**/*.ts".into()]),
        },
    );
    m.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let files = CursorHarness.cast_files(&m, src).unwrap();
    let rule_file = files.iter().find(|(p, _)| p.ends_with("r.mdc")).unwrap();
    let content: &str = &rule_file.1;
    assert!(
        content.contains("globs: src/**/*.ts, lib/**/*.ts"),
        "globs should be comma-separated, got:\n{content}"
    );
}
