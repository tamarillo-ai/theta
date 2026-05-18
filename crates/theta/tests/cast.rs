//! Cast (harness output) tests.

use std::collections::BTreeMap;

use theta_cast::{Caster, write_cast_output};
use theta_schema::{ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, minimal_manifest};

#[test]
fn cast_claude_code_produces_rules_without_synthetic_claude_md() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let output = dir.path().join("output");
    fs_err::create_dir_all(source.join("rules")).unwrap();
    fs_err::write(
        source.join("rules/safety.md"),
        "Never produce harmful output.",
    )
    .unwrap();

    let mut manifest = minimal_manifest("test-agent");
    manifest.agent.description = "Test agent".to_string();
    let mut rules = BTreeMap::new();
    rules.insert(
        "safety".to_string(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/safety.md")),
            summary: None,
            description: Some("Safety rule".to_string()),
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    manifest.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let caster = theta_cast::ClaudeCode;
    let files = caster.cast_files(&manifest, &source).unwrap();
    let written = write_cast_output(&files, &output).unwrap();

    assert_eq!(written.len(), 1);
    assert!(
        !output.join("CLAUDE.md").exists(),
        "CLAUDE.md must not be emitted when [instructions].system is unset"
    );
    let rule_file = fs_err::read_to_string(output.join(".claude/rules/safety.md")).unwrap();
    assert!(rule_file.contains("Never produce harmful output."));
}

#[test]
fn cast_copilot_produces_instructions_with_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let output = dir.path().join("output");
    fs_err::create_dir_all(source.join("rules")).unwrap();
    fs_err::write(
        source.join("rules/rust.md"),
        "Follow Rust 2024 edition conventions.",
    )
    .unwrap();

    let mut manifest = minimal_manifest("test-agent");
    manifest.agent.description = "Test agent".to_string();
    let mut rules = BTreeMap::new();
    rules.insert(
        "rust".to_string(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/rust.md")),
            summary: Some("this is a summary".into()),
            description: Some("Rust conventions".to_string()),
            apply: ApplyMode::Glob,
            apply_to: Some(vec!["**/*.rs".to_string()]),
        },
    );
    manifest.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let caster = theta_cast::Copilot;
    let files = caster.cast_files(&manifest, &source).unwrap();
    let written = write_cast_output(&files, &output).unwrap();

    assert_eq!(written.len(), 1);

    let content =
        fs_err::read_to_string(output.join(".github/instructions/rust.instructions.md")).unwrap();
    assert!(content.contains("description: Rust conventions"));
    assert!(content.contains("applyTo:"));
    assert!(content.contains("**/*.rs"));
    assert!(content.contains("Follow Rust 2024 edition conventions."));
}

#[test]
fn cast_cursor_produces_mdc_with_frontmatter() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let output = dir.path().join("output");
    fs_err::create_dir_all(source.join("rules")).unwrap();
    fs_err::write(source.join("rules/safety.md"), "Be safe.").unwrap();

    let mut manifest = minimal_manifest("test-agent");
    manifest.agent.description = "Test agent".to_string();
    let mut rules = BTreeMap::new();
    rules.insert(
        "safety".to_string(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/safety.md")),
            summary: None,
            description: Some("Safety".to_string()),
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    manifest.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let caster = theta_cast::CursorHarness;
    let files = caster.cast_files(&manifest, &source).unwrap();
    let written = write_cast_output(&files, &output).unwrap();

    assert_eq!(written.len(), 2);
    let system = fs_err::read_to_string(output.join(".cursor/rules/system.md")).unwrap();
    assert!(system.contains("alwaysApply: true"));
    let mdc = fs_err::read_to_string(output.join(".cursor/rules/safety.mdc")).unwrap();
    assert!(mdc.contains("alwaysApply: true"));
    assert!(mdc.contains("Be safe."));
}

#[test]
fn cast_source_and_output_are_independent() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("src");
    let output = dir.path().join("out");

    fs_err::create_dir_all(source.join("rules")).unwrap();
    fs_err::write(source.join("rules/r.md"), "rule content").unwrap();

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "Test".to_string();
    let mut rules = BTreeMap::new();
    rules.insert(
        "r".to_string(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/r.md")),
            summary: None,
            description: None,
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    manifest.instructions = Some(Instructions {
        system: None,
        rules: Some(rules),
    });

    let caster = theta_cast::ClaudeCode;
    let files = caster.cast_files(&manifest, &source).unwrap();
    let written = write_cast_output(&files, &output).unwrap();

    for path in &written {
        assert!(path.starts_with(&output));
        assert!(!path.starts_with(&source));
    }

    let rule_file = fs_err::read_to_string(output.join(".claude/rules/r.md")).unwrap();
    assert!(rule_file.contains("rule content"));
}
