//! Manifest validation tests.

use theta_manifest::{collect_document_diagnostics, create_manifest, read_document, read_manifest};
use theta_schema::{DiagLevel, Validate, minimal_manifest};
use theta_static::MANIFEST_FILE_NAME;

#[test]
fn fresh_manifest_produces_expected_warnings() {
    let manifest = minimal_manifest("test");
    let mut diags = Vec::new();
    manifest.validate(&mut diags);

    // placeholder description --> 1 warning
    assert_eq!(diags.len(), 1);
    assert!(diags.iter().all(|d| d.level == DiagLevel::Warn));
    assert!(diags.iter().any(|d| d.path == "[agent].description"));
}

#[test]
fn configured_manifest_produces_no_warnings() {
    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "OSINT people intelligence agent".to_string();
    manifest.agent.model = Some("claude-sonnet-4-20250514".to_string());

    let mut diags = Vec::new();
    manifest.validate(&mut diags);
    assert!(diags.is_empty());
}

#[test]
fn init_then_validate_full_pipeline() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(MANIFEST_FILE_NAME);

    let manifest = minimal_manifest("pipeline-test");
    create_manifest(&path, &manifest).unwrap();

    let doc = read_document(&path).unwrap();
    let parsed = read_manifest(&path).unwrap();

    let mut diags = Vec::new();
    collect_document_diagnostics(&doc, &mut diags);
    parsed.validate(&mut diags);

    assert!(!diags.iter().any(|d| d.path == "[root]"));
    assert_eq!(
        diags.iter().filter(|d| d.level == DiagLevel::Warn).count(),
        1
    );
}

#[test]
fn invalid_semver_produces_warning() {
    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "real description".to_string();
    manifest.agent.model = Some("gpt-4o".to_string());
    manifest.agent.version = Some("1.0.0-beta".to_string());

    let mut diags = Vec::new();
    manifest.validate(&mut diags);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].path == "[agent].version");
}

#[test]
fn bad_author_produces_warning() {
    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "real description".to_string();
    manifest.agent.model = Some("gpt-4o".to_string());
    manifest.agent.authors = Some(vec!["<broken>".to_string()]);

    let mut diags = Vec::new();
    manifest.validate(&mut diags);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].path == "[agent].authors");
}

#[test]
fn description_over_limit_produces_warning() {
    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "x".repeat(1025);
    manifest.agent.model = Some("gpt-4o".to_string());

    let mut diags = Vec::new();
    manifest.validate(&mut diags);
    assert!(diags.iter().any(|d| d.message.contains("exceeds")));
}

#[test]
fn unknown_section_produces_document_warning() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(MANIFEST_FILE_NAME);

    let manifest = minimal_manifest("test");
    create_manifest(&path, &manifest).unwrap();

    let mut content = fs_err::read_to_string(&path).unwrap();
    content.push_str("\n[bogus]\nkey = \"value\"\n");
    fs_err::write(&path, content).unwrap();

    let doc = read_document(&path).unwrap();
    let mut diags = Vec::new();
    collect_document_diagnostics(&doc, &mut diags);
    assert!(
        diags
            .iter()
            .any(|d| d.path == "[root]" && d.message.contains("bogus"))
    );
}

#[test]
fn unknown_field_in_section_rejects_at_parse_time() {
    let toml = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"
descripton = "typo"
"#;
    let result = toml::from_str::<theta_schema::ThetaManifest>(toml);
    assert!(result.is_err(), "typo in field name should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unknown field"),
        "error should mention 'unknown field', got: {err}"
    );
}

#[test]
fn unknown_top_level_section_rejects_at_parse_time() {
    let toml = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test agent"

[model]
default = "gpt-4"
"#;
    let result = toml::from_str::<theta_schema::ThetaManifest>(toml);
    assert!(result.is_err(), "[model] section should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unknown field"),
        "error should mention 'unknown field', got: {err}"
    );
}
