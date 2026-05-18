use super::*;

// JSONC parsing

#[test]
fn parse_jsonc_value_handles_trailing_commas() {
    let input = r#"{
        "servers": {
            "my-server": {
                "type": "stdio",
                "command": "my-cmd"
            },
        },
        "inputs": []
    }"#;
    let path = std::path::Path::new("test.json");
    let result = parse_jsonc_value(input, path);
    assert!(
        result.is_ok(),
        "trailing commas should be accepted: {result:?}"
    );
}

#[test]
fn parse_jsonc_map_handles_trailing_commas() {
    let input = r#"{"a": 1, "b": 2,}"#;
    let path = std::path::Path::new("test.json");
    let result = parse_jsonc_map(input, path);
    assert!(
        result.is_ok(),
        "trailing commas should be accepted: {result:?}"
    );
}

#[test]
fn parse_jsonc_map_rejects_non_object_root() {
    let input = r"[1, 2, 3]";
    let path = std::path::Path::new("test.json");
    let result = parse_jsonc_map(input, path);
    assert!(result.is_err());
    assert!(
        result.unwrap_err().to_string().contains("not an object"),
        "should mention the value is not an object"
    );
}

// frontmatter write/parse

#[test]
fn frontmatter_empty_entries_returns_empty_string() {
    assert_eq!(yaml_frontmatter(&[]).unwrap(), "");
}

#[test]
fn frontmatter_str_values_are_properly_escaped() {
    let fm =
        yaml_frontmatter(&[("description", fm_str("A rule with \"quotes\" and: colons"))]).unwrap();
    assert!(fm.starts_with("---\n"));
    assert!(fm.ends_with("---\n"));
    assert!(fm.contains("description:"));
    assert!(fm.contains("quotes"));
}

#[test]
fn frontmatter_bool_and_list() {
    let fm = yaml_frontmatter(&[
        ("alwaysApply", fm_bool(true)),
        ("globs", fm_list(&["*.rs".to_string(), "*.py".to_string()])),
    ])
    .unwrap();
    assert!(fm.contains("alwaysApply: true"));
    assert!(fm.contains("*.rs"));
    assert!(fm.contains("*.py"));
}

#[test]
fn frontmatter_round_trip_str() {
    let fm = yaml_frontmatter(&[("title", fm_str("hello world"))]).unwrap();
    let parsed = parse_frontmatter(&fm);
    assert_eq!(parsed.get_str("title"), Some("hello world"));
    assert!(
        parsed.content.trim().is_empty(),
        "body should be empty after frontmatter-only input"
    );
}

#[test]
fn frontmatter_round_trip_bool() {
    let fm = yaml_frontmatter(&[("alwaysApply", fm_bool(true))]).unwrap();
    let parsed = parse_frontmatter(&fm);
    assert_eq!(parsed.get_bool("alwaysApply"), Some(true));
}

#[test]
fn frontmatter_round_trip_list() {
    let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let fm = yaml_frontmatter(&[("tags", fm_list(&items))]).unwrap();
    let parsed = parse_frontmatter(&fm);
    assert_eq!(
        parsed.get_str_list("tags"),
        Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
    );
}

#[test]
fn frontmatter_round_trip_multiple_fields() {
    let fm = yaml_frontmatter(&[
        ("description", fm_str("my rule")),
        ("alwaysApply", fm_bool(false)),
        (
            "globs",
            fm_list(&["*.rs".to_string(), "*.toml".to_string()]),
        ),
    ])
    .unwrap();
    let parsed = parse_frontmatter(&fm);
    assert_eq!(parsed.get_str("description"), Some("my rule"));
    assert_eq!(parsed.get_bool("alwaysApply"), Some(false));
    assert_eq!(
        parsed.get_str_list("globs"),
        Some(vec!["*.rs".to_string(), "*.toml".to_string()])
    );
}

#[test]
fn frontmatter_round_trip_with_body() {
    let fm = yaml_frontmatter(&[("title", fm_str("test"))]).unwrap();
    let full = format!("{fm}this is the body\n");
    let parsed = parse_frontmatter(&full);
    assert_eq!(parsed.get_str("title"), Some("test"));
    assert!(parsed.content.contains("this is the body"));
}

#[test]
fn parse_frontmatter_extracts_yaml() {
    let input = "---\ntitle: hello\nalwaysApply: true\n---\n\nbody text";
    let parsed = parse_frontmatter(input);
    assert_eq!(parsed.get_str("title"), Some("hello"));
    assert_eq!(parsed.get_bool("alwaysApply"), Some(true));
    assert!(parsed.content.contains("body text"));
}

#[test]
fn parse_frontmatter_no_frontmatter_returns_full_content() {
    let input = "just some text\nno frontmatter here";
    let parsed = parse_frontmatter(input);
    assert!(parsed.data.is_empty());
    assert_eq!(parsed.content, input);
}

#[test]
fn parse_frontmatter_with_list() {
    let input = "---\nglobs:\n  - '*.rs'\n  - '*.py'\n---\n\ncontent";
    let parsed = parse_frontmatter(input);
    let globs = parsed.get_str_list("globs").unwrap();
    assert_eq!(globs, vec!["*.rs", "*.py"]);
}

#[test]
fn parse_frontmatter_unquoted_glob_asterisk() {
    // Cursor documents unquoted globs: `globs: *.ts`, `globs: src/**/*.tsx`
    // ref: https://cursor.com/docs/rules#glob-pattern-examples
    //
    // `*` is the YAML alias indicator (c-alias) and cannot start a plain scalar
    // ref: https://yaml.org/spec/1.2.2/ §5.3 - [14] c-alias ::= '*'
    // ref: https://yaml.org/spec/1.2.2/ §7.3.3 - plain scalars exclude c-indicator
    //
    // serde_norway (and any spec-compliant YAML parser) rejects `globs: *.astro`
    // as invalid. Verify we degrade gracefully: body is extracted without
    // double-frontmatter corruption, even though frontmatter data is lost
    let input = "---\ndescription: \nglobs: *.astro\nalwaysApply: false\n---\n### Guidelines";
    let parsed = parse_frontmatter(input);
    // if YAML parse fails, body should still be correct (no double frontmatter)
    assert!(
        parsed.content.contains("Guidelines"),
        "body must not include frontmatter block"
    );
    assert!(
        !parsed.content.contains("---"),
        "body must not contain frontmatter delimiters"
    );
    assert!(parsed.data.is_empty(), "failed parse must yield empty data");
}

#[test]
fn parse_frontmatter_empty_yaml_block() {
    let input = "---\n---\nbody only";
    let parsed = parse_frontmatter(input);
    assert!(parsed.data.is_empty());
    assert!(parsed.content.contains("body only"));
}

// JSON <-> TOML conversion

#[test]
fn json_to_toml_round_trip() {
    let json = serde_json::json!({
        "name": "test",
        "count": 42,
        "enabled": true,
        "tags": ["a", "b"]
    });
    let toml_str = json_to_toml_string(&json).unwrap();
    assert!(toml_str.contains("name = \"test\""));
    assert!(toml_str.contains("count = 42"));
    assert!(toml_str.contains("enabled = true"));
}

#[test]
fn toml_to_json_round_trip() {
    let toml_str = "name = \"test\"\ncount = 42\n";
    let json = toml_str_to_json(toml_str).unwrap();
    assert_eq!(json["name"], "test");
    assert_eq!(json["count"], 42);
}

#[test]
fn json_to_toml_item_scalar_string() {
    let item = json_to_toml_item(&serde_json::json!("hello"));
    assert_eq!(item.as_str(), Some("hello"));
}

#[test]
fn json_to_toml_item_scalar_bool() {
    let item = json_to_toml_item(&serde_json::json!(true));
    assert_eq!(item.as_bool(), Some(true));
}

#[test]
fn json_to_toml_item_scalar_integer() {
    let item = json_to_toml_item(&serde_json::json!(42));
    assert_eq!(item.as_integer(), Some(42));
}

#[test]
fn json_to_toml_item_scalar_float() {
    let item = json_to_toml_item(&serde_json::json!(std::f64::consts::PI));
    assert!(item.as_float().is_some());
}

#[test]
fn json_to_toml_item_null_is_none() {
    let item = json_to_toml_item(&serde_json::json!(null));
    assert!(item.is_none());
}

#[test]
fn json_to_toml_item_array_of_strings() {
    let item = json_to_toml_item(&serde_json::json!(["a", "b", "c"]));
    let arr = item.as_value().and_then(|v| v.as_array()).unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr.get(0).and_then(|v| v.as_str()), Some("a"));
}

#[test]
fn json_to_toml_item_array_of_integers() {
    let item = json_to_toml_item(&serde_json::json!([1, 2, 3]));
    let arr = item.as_value().and_then(|v| v.as_array()).unwrap();
    assert_eq!(arr.len(), 3);
}

#[test]
fn json_to_toml_item_array_with_nulls_strips_them() {
    let item = json_to_toml_item(&serde_json::json!([1, null, 3]));
    let arr = item.as_value().and_then(|v| v.as_array()).unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn json_to_toml_item_empty_array() {
    let item = json_to_toml_item(&serde_json::json!([]));
    let arr = item.as_value().and_then(|v| v.as_array()).unwrap();
    assert_eq!(arr.len(), 0);
}

#[test]
fn json_to_toml_item_object_becomes_table() {
    let item = json_to_toml_item(&serde_json::json!({"a": 1, "b": "two"}));
    let tbl = item.as_table().unwrap();
    assert_eq!(tbl.get("a").and_then(toml_edit::Item::as_integer), Some(1));
    assert_eq!(tbl.get("b").and_then(|i| i.as_str()), Some("two"));
}

#[test]
fn json_to_toml_item_nested_object() {
    let input = serde_json::json!({"outer": {"inner": true}});
    let item = json_to_toml_item(&input);
    let outer = item.as_table().unwrap();
    let inner = outer.get("outer").and_then(|i| i.as_table()).unwrap();
    assert_eq!(
        inner.get("inner").and_then(toml_edit::Item::as_bool),
        Some(true)
    );
}

#[test]
fn json_to_toml_item_object_with_array_value() {
    let input = serde_json::json!({"tags": ["a", "b"]});
    let item = json_to_toml_item(&input);
    let tbl = item.as_table().unwrap();
    let arr = tbl
        .get("tags")
        .and_then(|i| i.as_value())
        .and_then(|v| v.as_array())
        .unwrap();
    assert_eq!(arr.len(), 2);
}

#[test]
fn json_to_toml_item_mixed_type_array() {
    let item = json_to_toml_item(&serde_json::json!([1, "two", true]));
    let arr = item.as_value().and_then(|v| v.as_array()).unwrap();
    assert_eq!(arr.len(), 3);
}

#[test]
fn json_to_toml_item_survives_nested_null() {
    let input = serde_json::json!({"sandbox": {"enabled": true, "timeout": null}});
    let item = json_to_toml_item(&input);
    assert!(
        !item.is_none(),
        "object with nested null should not be dropped"
    );
}

#[test]
fn json_to_toml_item_with_diagnostics_reports_nulls() {
    let input = serde_json::json!({"keep": 1, "drop_me": null});
    let mut diags = Vec::new();
    let item = json_to_toml_item_with_diagnostics(&input, "test.field", &mut diags);
    assert!(!item.is_none());
    assert!(
        diags.iter().any(|d| d.message.contains("null")),
        "should emit diagnostic for null: {diags:?}"
    );
}

#[test]
fn json_to_toml_item_with_diagnostics_clean_value_no_diags() {
    let input = serde_json::json!({"a": 1, "b": "two"});
    let mut diags = Vec::new();
    let item = json_to_toml_item_with_diagnostics(&input, "test.field", &mut diags);
    assert!(!item.is_none());
    assert!(diags.is_empty(), "clean value should emit no diagnostics");
}

// strip_json_nulls

#[test]
fn strip_json_nulls_removes_null_fields() {
    let input = serde_json::json!({"a": 1, "b": null, "c": "hello"});
    let result = strip_json_nulls(&input);
    assert_eq!(result, serde_json::json!({"a": 1, "c": "hello"}));
}

#[test]
fn strip_json_nulls_removes_nested_nulls() {
    let input = serde_json::json!({"outer": {"keep": true, "drop": null}});
    let result = strip_json_nulls(&input);
    assert_eq!(result, serde_json::json!({"outer": {"keep": true}}));
}

#[test]
fn strip_json_nulls_removes_nulls_in_arrays() {
    let input = serde_json::json!([1, null, 3]);
    let result = strip_json_nulls(&input);
    assert_eq!(result, serde_json::json!([1, 3]));
}

#[test]
fn strip_json_nulls_preserves_non_null_values() {
    let input = serde_json::json!({"a": 1, "b": true, "c": [1, 2]});
    let result = strip_json_nulls(&input);
    assert_eq!(result, input);
}

#[test]
fn strip_json_nulls_idempotent() {
    let input = serde_json::json!({"a": 1, "b": null, "c": {"d": null, "e": 2}});
    let once = strip_json_nulls(&input);
    let twice = strip_json_nulls(&once);
    assert_eq!(once, twice, "strip_json_nulls should be idempotent");
}

#[test]
fn strip_json_nulls_all_null_object_becomes_empty() {
    let input = serde_json::json!({"a": null, "b": null});
    let result = strip_json_nulls(&input);
    assert_eq!(result, serde_json::json!({}));
}

#[test]
fn strip_json_nulls_tracked_reports_paths() {
    let input = serde_json::json!({"a": {"b": null}, "c": null});
    let (_, paths) = strip_json_nulls_tracked(&input, "root");
    assert!(paths.contains(&"root.a.b".to_string()));
    assert!(paths.contains(&"root.c".to_string()));
}

// identity header

#[test]
fn strip_identity_header_extracts_name_and_desc() {
    let content = "# My Agent\n\nthis is the description\n\nthe actual body";
    let (name, desc, body, _) = strip_identity_header_with_shape(content);
    assert_eq!(name.as_deref(), Some("My Agent"));
    assert_eq!(desc.as_deref(), Some("this is the description"));
    assert_eq!(body, "the actual body");
}

#[test]
fn strip_identity_header_no_header() {
    let content = "just body text";
    let (name, desc, body, _) = strip_identity_header_with_shape(content);
    assert!(name.is_none());
    assert!(desc.is_none());
    assert_eq!(body, "just body text");
}

#[test]
fn strip_identity_header_shape_well_formed() {
    let content = "# My Agent\n\nthis is the description\n\nthe actual body";
    let (_, _, _, shape) = strip_identity_header_with_shape(content);
    assert_eq!(shape, IdentityHeaderShape::WellFormed);
}

#[test]
fn strip_identity_header_shape_no_heading() {
    let content = "just body text";
    let (_, _, _, shape) = strip_identity_header_with_shape(content);
    assert_eq!(shape, IdentityHeaderShape::NoHeading);
}

#[test]
fn strip_identity_header_shape_heading_only() {
    let content = "# Just A Heading\n";
    let (name, desc, body, shape) = strip_identity_header_with_shape(content);
    assert_eq!(name.as_deref(), Some("Just A Heading"));
    assert!(desc.is_none());
    assert!(body.is_empty());
    assert_eq!(shape, IdentityHeaderShape::HeadingOnly);
}

#[test]
fn strip_identity_header_shape_heading_no_blank_line() {
    let content = "# Agent\nimmediately after heading no blank line body content\n";
    let (name, desc, body, shape) = strip_identity_header_with_shape(content);
    assert_eq!(name.as_deref(), Some("Agent"));
    assert!(desc.is_some());
    assert!(body.is_empty());
    assert_eq!(shape, IdentityHeaderShape::HeadingNoBlankLine);
}

// section ordering

#[test]
fn section_order_covers_all_manifest_sections() {
    let expected = [
        "theta",
        "agent",
        "instructions",
        "skills",
        "tools",
        "subagents",
        "harness",
        "extras",
    ];
    assert_eq!(
        theta_static::MANIFEST_SECTION_ORDER,
        &expected,
        "MANIFEST_SECTION_ORDER drifted from canonical manifest structure"
    );
}

#[test]
fn reorder_fixes_scrambled_document() {
    let mut doc = toml_edit::DocumentMut::new();
    doc["harness"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["tools"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["skills"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["instructions"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["agent"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["theta"] = toml_edit::Item::Table(toml_edit::Table::new());

    reorder_import_document(&mut doc);

    let keys: Vec<&str> = doc.as_table().iter().map(|(k, _)| k).collect();
    assert_eq!(
        keys,
        vec![
            "theta",
            "agent",
            "instructions",
            "skills",
            "tools",
            "harness"
        ]
    );
}

#[test]
fn reorder_preserves_unknown_sections_at_end() {
    let mut doc = toml_edit::DocumentMut::new();
    doc["custom"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["theta"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["agent"] = toml_edit::Item::Table(toml_edit::Table::new());

    reorder_import_document(&mut doc);

    let keys: Vec<&str> = doc.as_table().iter().map(|(k, _)| k).collect();
    assert_eq!(keys, vec!["theta", "agent", "custom"]);
}

#[test]
fn reorder_subset_of_sections() {
    let mut doc = toml_edit::DocumentMut::new();
    doc["tools"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["theta"] = toml_edit::Item::Table(toml_edit::Table::new());
    doc["agent"] = toml_edit::Item::Table(toml_edit::Table::new());

    reorder_import_document(&mut doc);

    let keys: Vec<&str> = doc.as_table().iter().map(|(k, _)| k).collect();
    assert_eq!(keys, vec!["theta", "agent", "tools"]);
}

// merge

#[test]
fn merge_json_objects_overlay_wins() {
    let base = serde_json::json!({"a": 1, "b": 2})
        .as_object()
        .unwrap()
        .clone();
    let overlay = serde_json::json!({"b": 99, "c": 3})
        .as_object()
        .unwrap()
        .clone();
    let mut diags = Vec::new();
    let merged = merge_json_objects(base, overlay, "ctx", &mut diags);
    assert_eq!(merged["a"], serde_json::json!(1));
    assert_eq!(merged["b"], serde_json::json!(99));
    assert_eq!(merged["c"], serde_json::json!(3));
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].level, theta_schema::DiagLevel::Warn);
    assert!(diags[0].path.contains('b'));
}

#[test]
fn merge_json_objects_no_conflict_no_diags() {
    let base = serde_json::json!({"a": 1}).as_object().unwrap().clone();
    let overlay = serde_json::json!({"b": 2}).as_object().unwrap().clone();
    let mut diags = Vec::new();
    let merged = merge_json_objects(base, overlay, "ctx", &mut diags);
    assert_eq!(merged.len(), 2);
    assert!(diags.is_empty());
}

#[test]
fn merge_json_objects_equal_value_no_diag() {
    let base = serde_json::json!({"a": 1}).as_object().unwrap().clone();
    let overlay = serde_json::json!({"a": 1}).as_object().unwrap().clone();
    let mut diags = Vec::new();
    let _ = merge_json_objects(base, overlay, "ctx", &mut diags);
    assert!(diags.is_empty());
}

// misc

#[test]
fn default_agent_name_uses_dirname() {
    let dir = PathBuf::from("/home/user/my-project");
    assert_eq!(default_agent_name(&dir), "my-project-agent");
}
// kebab_case

#[test]
fn kebab_case_lowercases_and_splits_on_case_boundary() {
    assert_eq!(kebab_case("MyAgent"), "my-agent");
}

#[test]
fn kebab_case_replaces_spaces_and_underscores() {
    assert_eq!(kebab_case("my agent_name"), "my-agent-name");
}

#[test]
fn kebab_case_strips_leading_trailing_hyphens() {
    assert_eq!(kebab_case("-leading-"), "leading");
}

#[test]
fn kebab_case_collapses_consecutive_separators() {
    assert_eq!(kebab_case("a  b__c"), "a-b-c");
}

#[test]
fn kebab_case_empty_string() {
    assert_eq!(kebab_case(""), "");
}

// read_existing_json_map

#[test]
fn read_existing_json_map_missing_file_returns_ok_none() {
    let result = read_existing_json_map(Path::new("/nonexistent/path.json"));
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn read_existing_json_map_valid_file_returns_map() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.json");
    fs_err::write(&path, r#"{"key": "value"}"#).unwrap();
    let result = read_existing_json_map(&path).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap()["key"], serde_json::json!("value"));
}

#[test]
fn read_existing_json_map_corrupt_file_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    fs_err::write(&path, "not json at all {{{").unwrap();
    let result = read_existing_json_map(&path);
    assert!(
        result.is_err(),
        "corrupt file should propagate error, not return None"
    );
}

// split_frontmatter edge cases

#[test]
fn split_frontmatter_no_closing_delimiter() {
    let input = "---\ntitle: hello\nbody without closing";
    let parsed = parse_frontmatter(input);
    assert!(
        parsed.data.is_empty(),
        "unclosed frontmatter should be treated as plain content"
    );
}

#[test]
fn split_frontmatter_only_dashes_no_content() {
    let input = "---\n---\n";
    let parsed = parse_frontmatter(input);
    assert!(parsed.data.is_empty());
}

// json_to_toml_item edge cases

#[test]
fn json_to_toml_item_empty_object() {
    let item = json_to_toml_item(&serde_json::json!({}));
    let tbl = item.as_table().unwrap();
    assert!(tbl.is_empty());
}

#[test]
fn json_to_toml_item_deeply_nested() {
    let input = serde_json::json!({"a": {"b": {"c": {"d": 42}}}});
    let item = json_to_toml_item(&input);
    let a = item.as_table().unwrap();
    let b = a.get("a").and_then(|i| i.as_table()).unwrap();
    let c = b.get("b").and_then(|i| i.as_table()).unwrap();
    let d = c.get("c").and_then(|i| i.as_table()).unwrap();
    assert_eq!(d.get("d").and_then(toml_edit::Item::as_integer), Some(42));
}

#[test]
fn json_to_toml_item_array_of_objects() {
    let input = serde_json::json!([{"a": 1}, {"b": 2}]);
    let item = json_to_toml_item(&input);
    let arr = item.as_value().and_then(|v| v.as_array()).unwrap();
    assert_eq!(arr.len(), 2);
}
#[test]
fn parse_frontmatter_colon_in_value_degrades() {
    // Cursor allows unquoted colons: `description: Go patterns: Repository`
    // YAML sees `: ` as a mapping indicator and rejects it
    let input = "---\ndescription: Go patterns: Repository, Adapter\nglobs: [\"**/*.go\"]\nalwaysApply: false\n---\nbody text";
    let parsed = parse_frontmatter(input);
    assert!(parsed.data.is_empty(), "failed parse must yield empty data");
    assert_eq!(parsed.content, "body text", "body must be preserved");
}

#[test]
fn parse_frontmatter_valid_yaml() {
    let input =
        "---\ndescription: Clean rules\nglobs: [\"**/*.go\"]\nalwaysApply: false\n---\nbody text";
    let parsed = parse_frontmatter(input);
    assert_eq!(parsed.get_str("description"), Some("Clean rules"));
    assert_eq!(parsed.get_bool("alwaysApply"), Some(false));
    assert!(parsed.get_str_list("globs").is_some());
    assert_eq!(parsed.content, "body text");
}

#[test]
fn parse_frontmatter_no_frontmatter() {
    let input = "just plain text\nno frontmatter here";
    let parsed = parse_frontmatter(input);
    assert!(parsed.data.is_empty());
}
