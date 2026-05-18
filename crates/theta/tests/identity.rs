//! Identity + describe tests.

use theta_manifest::{
    create_manifest, mutate_manifest, read_document, read_manifest, schema_version,
    set_value_strict,
};
use theta_schema::{minimal_manifest, normalize_agent_name};
use theta_static::{DEFAULT_DESCRIPTION, DEFAULT_VERSION, MANIFEST_FILE_NAME};

// assert that a TOML mutation changed ONLY the expected key path
fn assert_only_section_changed(before: &str, after: &str, changed_section: &str) {
    let doc_before: toml_edit::DocumentMut = before.parse().unwrap();
    let doc_after: toml_edit::DocumentMut = after.parse().unwrap();

    for (key, item_before) in doc_before.as_table() {
        if key == changed_section {
            continue;
        }
        let item_after = doc_after
            .as_table()
            .get(key)
            .unwrap_or_else(|| panic!("section [{key}] disappeared after mutation"));
        assert_eq!(
            item_before.to_string(),
            item_after.to_string(),
            "section [{key}] changed unexpectedly"
        );
    }
}

#[test]
fn init_produces_round_trippable_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(MANIFEST_FILE_NAME);

    let manifest = minimal_manifest("test-agent");
    create_manifest(&path, &manifest).unwrap();

    let parsed = read_manifest(&path).unwrap();
    assert_eq!(parsed.agent.name, "test-agent");
    assert_eq!(parsed.agent.description, DEFAULT_DESCRIPTION);
    assert_eq!(parsed.agent.version.as_deref(), Some(DEFAULT_VERSION));

    let doc = read_document(&path).unwrap();
    assert!(schema_version(&doc).is_ok());
}

#[test]
fn normalize_from_directory_style_paths() {
    assert_eq!(normalize_agent_name("my-project"), "my-project");
    assert_eq!(normalize_agent_name("My Cool Agent"), "my-cool-agent");
    assert_eq!(normalize_agent_name("code_reviewer"), "code-reviewer");
    assert_eq!(normalize_agent_name(""), "my-agent");
    assert_eq!(normalize_agent_name("---"), "my-agent");
}

#[test]
fn describe_read_returns_current_description() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(MANIFEST_FILE_NAME);

    let mut manifest = minimal_manifest("test");
    manifest.agent.description = "OSINT investigator".to_string();
    create_manifest(&path, &manifest).unwrap();

    let parsed = read_manifest(&path).unwrap();
    assert_eq!(parsed.agent.description, "OSINT investigator");
}

#[test]
fn describe_write_updates_description_preserving_format() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(MANIFEST_FILE_NAME);

    let manifest = minimal_manifest("test");
    create_manifest(&path, &manifest).unwrap();

    let before = fs_err::read_to_string(&path).unwrap();

    mutate_manifest(&path, |doc| {
        set_value_strict(
            doc,
            &["agent", "description"],
            toml_edit::Value::from("OSINT people intelligence agent"),
        )
        .unwrap();
    })
    .unwrap();

    let after = fs_err::read_to_string(&path).unwrap();

    assert_only_section_changed(&before, &after, "agent");

    let parsed = read_manifest(&path).unwrap();
    assert_eq!(parsed.agent.description, "OSINT people intelligence agent");
}

#[test]
fn describe_write_rejects_over_limit() {
    let long_desc = "x".repeat(1025);
    assert!(long_desc.len() > theta_static::MAX_DESCRIPTION_LENGTH);
}

#[test]
fn describe_read_detects_placeholder() {
    let manifest = minimal_manifest("test");
    assert!(theta_static::is_placeholder_description(
        &manifest.agent.description
    ));
}
