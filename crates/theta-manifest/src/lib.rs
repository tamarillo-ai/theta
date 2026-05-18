//! Formatting-preserving `theta.toml` read/write/mutate via `toml_edit`.

use std::path::{Path, PathBuf};

use theta_schema::{Diagnostic, ThetaManifest};
use toml_edit::DocumentMut;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ManifestError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("table does not exist: {table} (within path: {parents:?})")]
    TableDoesNotExist { table: String, parents: Vec<String> },

    #[error("failed to parse {path} as TOML: {source}")]
    ParseEdit {
        path: PathBuf,
        source: toml_edit::TomlError,
    },

    #[error("failed to deserialize {path}: {source}")]
    Deserialize {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("failed to deserialize manifest: {source}")]
    DeserializeStr { source: toml::de::Error },

    #[error("failed to serialize manifest: {source}")]
    Serialize { source: toml::ser::Error },

    #[error("{path} already exists")]
    AlreadyExists { path: PathBuf },

    #[error("[theta].schema is missing - add `schema = \"{expected}\"` to [theta]")]
    MissingSchemaVersion { expected: String },

    #[error("unknown schema version \"{found}\" - supported versions: {supported}")]
    InvalidSchemaVersion { found: String, supported: String },

    #[error(
        "unknown top-level manifest section `{section}` - expected one of: {expected}. \
         this is a bug in a theta importer; the section list lives in \
         `theta_static::MANIFEST_SECTION_ORDER`"
    )]
    UnknownSection { section: String, expected: String },
}

pub fn schema_version(doc: &DocumentMut) -> Result<&str, ManifestError> {
    let version = doc
        .get("theta")
        .and_then(|t| t.get("schema"))
        .and_then(|v| v.as_str());

    match version {
        None => Err(ManifestError::MissingSchemaVersion {
            expected: theta_static::SCHEMA_VERSION.to_string(),
        }),
        Some(v) if theta_static::SCHEMA_VERSIONS.contains(&v) => Ok(v),
        Some(v) => Err(ManifestError::InvalidSchemaVersion {
            found: v.to_string(),
            supported: theta_static::SCHEMA_VERSIONS.join(", "),
        }),
    }
}

pub fn parse_manifest(content: &str) -> Result<ThetaManifest, ManifestError> {
    toml::from_str(content).map_err(|e| ManifestError::DeserializeStr { source: e })
}

pub fn read_manifest(path: &Path) -> Result<ThetaManifest, ManifestError> {
    let content = fs_err::read_to_string(path).map_err(|e| ManifestError::Read {
        path: path.to_path_buf(),
        source: e,
    })?;
    toml::from_str(&content).map_err(|e| ManifestError::Deserialize {
        path: path.to_path_buf(),
        source: e,
    })
}

pub fn read_document(path: &Path) -> Result<DocumentMut, ManifestError> {
    let content = fs_err::read_to_string(path).map_err(|e| ManifestError::Read {
        path: path.to_path_buf(),
        source: e,
    })?;
    content
        .parse::<DocumentMut>()
        .map_err(|e| ManifestError::ParseEdit {
            path: path.to_path_buf(),
            source: e,
        })
}

pub fn write_document(path: &Path, doc: &DocumentMut) -> Result<(), ManifestError> {
    fs_err::write(path, doc.to_string()).map_err(|e| ManifestError::Write {
        path: path.to_path_buf(),
        source: e,
    })
}

pub fn mutate_manifest(path: &Path, f: impl FnOnce(&mut DocumentMut)) -> Result<(), ManifestError> {
    let mut doc = read_document(path)?;
    f(&mut doc);
    write_document(path, &doc)
}

pub fn serialize_manifest(manifest: &ThetaManifest) -> Result<String, ManifestError> {
    toml::to_string_pretty(manifest).map_err(|e| ManifestError::Serialize { source: e })
}

pub fn create_manifest(path: &Path, manifest: &ThetaManifest) -> Result<(), ManifestError> {
    if path.exists() {
        return Err(ManifestError::AlreadyExists {
            path: path.to_path_buf(),
        });
    }
    let content = serialize_manifest(manifest)?;
    fs_err::write(path, content).map_err(|e| ManifestError::Write {
        path: path.to_path_buf(),
        source: e,
    })
}

pub fn set_value(
    doc: &mut DocumentMut,
    keys: &[&str],
    value: toml_edit::Value,
) -> Result<(), ManifestError> {
    set_value_inner(doc, keys, value, true)
}

pub fn set_value_strict(
    doc: &mut DocumentMut,
    keys: &[&str],
    value: toml_edit::Value,
) -> Result<(), ManifestError> {
    set_value_inner(doc, keys, value, false)
}

fn set_value_inner(
    doc: &mut DocumentMut,
    keys: &[&str],
    value: toml_edit::Value,
    create_parents: bool,
) -> Result<(), ManifestError> {
    assert!(!keys.is_empty(), "key path must not be empty");

    if keys.len() == 1 {
        doc[keys[0]] = toml_edit::Item::Value(value);
        return Ok(());
    }

    let (parents, leaf) = keys.split_at(keys.len() - 1);
    let mut table = doc.as_table_mut();
    for &key in parents {
        if !table.contains_key(key) {
            if !create_parents {
                return Err(ManifestError::TableDoesNotExist {
                    table: key.to_string(),
                    parents: parents
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect(),
                });
            }
            table[key] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        table = table[key]
            .as_table_mut()
            .ok_or_else(|| ManifestError::TableDoesNotExist {
                table: key.to_string(),
                parents: parents
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            })?;
    }
    table[leaf[0]] = toml_edit::Item::Value(value);
    Ok(())
}

pub fn has_rule(doc: &DocumentMut, name: &str) -> bool {
    doc.get("instructions")
        .and_then(|i| i.get("rules"))
        .and_then(|r| r.get(name))
        .is_some()
}

pub fn has_skill(doc: &DocumentMut, name: &str) -> bool {
    doc.get("skills").and_then(|s| s.get(name)).is_some()
}

pub fn has_tool(doc: &DocumentMut, name: &str) -> bool {
    doc.get("tools").and_then(|t| t.get(name)).is_some()
}

pub fn ensure_table<'a>(doc: &'a mut DocumentMut, keys: &[&str]) -> &'a mut toml_edit::Table {
    let mut table = doc.as_table_mut();
    for &key in keys {
        if !table.contains_key(key) {
            let mut t = toml_edit::Table::new();
            t.set_implicit(true);
            table[key] = toml_edit::Item::Table(t);
        }
        table = table[key].as_table_mut().expect("key is not a table");
    }
    table
}

/// Ensure `[instructions].system` is set in the document.
/// Creates the `[instructions]` section if it doesn't exist.
pub fn set_system_path(doc: &mut DocumentMut) {
    if doc.get("instructions").is_none() {
        let mut instructions = toml_edit::Table::new();
        instructions["system"] = toml_edit::value(theta_static::SYSTEM_FILE_NAME);
        doc["instructions"] = toml_edit::Item::Table(instructions);
    } else if doc["instructions"].get("system").is_none() {
        doc["instructions"]["system"] = toml_edit::value(theta_static::SYSTEM_FILE_NAME);
    }
}

/// Set `[instructions].system` to a specific relative path.
pub fn set_system_path_value(doc: &mut DocumentMut, rel_path: &str) {
    let table = ensure_table(doc, &["instructions"]);
    table["system"] = toml_edit::value(rel_path);
}

/// Remove `[instructions].system` and clean up empty `[instructions]`.
pub fn remove_system(doc: &mut DocumentMut) {
    if let Some(instructions) = doc.get_mut("instructions").and_then(|i| i.as_table_mut()) {
        instructions.remove("system");
        if instructions.is_empty() {
            doc.as_table_mut().remove("instructions");
        }
    }
}

/// Set `[instructions.rules.<name>]` to the given table.
pub fn set_rule(doc: &mut DocumentMut, name: &str, rule: toml_edit::Table) {
    let rules_table = ensure_table(doc, &["instructions", "rules"]);
    rules_table[name] = toml_edit::Item::Table(rule);
}

/// Set `[skills.<name>]` to the given table.
pub fn set_skill(doc: &mut DocumentMut, name: &str, skill: toml_edit::Table) {
    let skills_table = ensure_table(doc, &["skills"]);
    skills_table[name] = toml_edit::Item::Table(skill);
}

/// Build a `[skills.<name>]` table with `source = { path = "skills/<name>" }`
/// pointing at the canonical theta-local skill directory.
///
/// This is the standard form used by every harness importer when copying a
/// skill folder into the theta tree. The relative path string is built via
/// `theta_static::ThetaProjectLayout::skill_rel(name)`.
pub fn local_skill_entry(name: &str) -> toml_edit::Table {
    let mut skill = toml_edit::Table::new();
    let mut source = toml_edit::InlineTable::new();
    source.insert(
        "path",
        toml_edit::Value::from(theta_static::ThetaProjectLayout::skill_rel(name)),
    );
    skill["source"] = toml_edit::Item::Value(toml_edit::Value::InlineTable(source));
    skill
}

/// Set `[skills.<name>]` directly on the document with the canonical local
/// source-path entry. Convenience wrapper for single-skill importers; bulk
/// importers should aggregate via `local_skill_entry` and call
/// `set_section(doc, "skills", ...)` once at the end.
pub fn set_local_skill(doc: &mut DocumentMut, name: &str) {
    set_skill(doc, name, local_skill_entry(name));
}

/// Set `[tools.<name>]` to the given table.
pub fn set_tool(doc: &mut DocumentMut, name: &str, tool: toml_edit::Table) {
    let tools_table = ensure_table(doc, &["tools"]);
    tools_table[name] = toml_edit::Item::Table(tool);
}

/// Append a `[[subagents]]` entry.
pub fn append_subagent(doc: &mut DocumentMut, entry: toml_edit::Table) {
    if !doc.contains_key("subagents") {
        doc["subagents"] = toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new());
    }
    if let Some(arr) = doc["subagents"].as_array_of_tables_mut() {
        arr.push(entry);
    }
}

/// Set a top-level theta.toml section wholesale.
///
/// `key` must be one of the canonical sections listed in
/// `theta_static::MANIFEST_SECTION_ORDER`. Callers receive
/// `ManifestError::UnknownSection` for any other key, so importers fail
/// loudly on a typo or a stale section name rather than writing a junk
/// top-level table into the user's manifest.
pub fn set_section(
    doc: &mut DocumentMut,
    key: &str,
    table: toml_edit::Table,
) -> Result<(), ManifestError> {
    if !theta_static::is_known_section(key) {
        return Err(ManifestError::UnknownSection {
            section: key.to_string(),
            expected: theta_static::MANIFEST_SECTION_ORDER.join(", "),
        });
    }
    doc[key] = toml_edit::Item::Table(table);
    Ok(())
}

/// Set `[[subagents]]` from a complete array of tables (for importers).
pub fn set_subagents(doc: &mut DocumentMut, arr: toml_edit::ArrayOfTables) {
    doc["subagents"] = toml_edit::Item::ArrayOfTables(arr);
}

/// Set `[instructions.rules]` from a complete table (for importers).
pub fn set_rules_section(doc: &mut DocumentMut, rules: toml_edit::Table) {
    let _ = ensure_table(doc, &["instructions"]);
    doc["instructions"]["rules"] = toml_edit::Item::Table(rules);
}

/// Merge additional rules into existing `[instructions.rules]`, skipping duplicates.
pub fn merge_rules(doc: &mut DocumentMut, additional: toml_edit::Table) {
    let rules_table = ensure_table(doc, &["instructions", "rules"]);
    for (k, v) in &additional {
        if !rules_table.contains_key(k) {
            rules_table.insert(k, v.clone());
        }
    }
}

pub fn collect_document_diagnostics(doc: &DocumentMut, diags: &mut Vec<Diagnostic>) {
    let known = theta_schema::known_sections();
    for (key, _) in doc.as_table() {
        if !known.iter().any(|k| k == key) {
            diags.push(Diagnostic::warn(
                "[root]",
                format!("unknown section [{key}]"),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_value_creates_intermediate_tables() {
        let mut doc = DocumentMut::new();
        set_value(
            &mut doc,
            &["model", "default"],
            toml_edit::Value::from("claude-sonnet-4-20250514"),
        )
        .unwrap();
        assert_eq!(
            doc["model"]["default"].as_str(),
            Some("claude-sonnet-4-20250514")
        );
    }

    #[test]
    fn set_value_single_key() {
        let mut doc = DocumentMut::new();
        set_value(&mut doc, &["name"], toml_edit::Value::from("test")).unwrap();
        assert_eq!(doc["name"].as_str(), Some("test"));
    }

    #[test]
    fn set_value_strict_fails_on_missing_parent() {
        let mut doc = DocumentMut::new();
        let result = set_value_strict(
            &mut doc,
            &["model", "default"],
            toml_edit::Value::from("sonnet"),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ManifestError::TableDoesNotExist { table, parents } => {
                assert_eq!(table, "model");
                assert_eq!(parents, vec!["model"]);
            }
            other => panic!("expected TableDoesNotExist, got: {other}"),
        }
    }

    #[test]
    fn set_value_strict_works_when_parent_exists() {
        let mut doc = DocumentMut::new();
        set_value(
            &mut doc,
            &["model", "default"],
            toml_edit::Value::from("placeholder"),
        )
        .unwrap();
        set_value_strict(
            &mut doc,
            &["model", "reasoning_effort"],
            toml_edit::Value::from("high"),
        )
        .unwrap();
        assert_eq!(doc["model"]["reasoning_effort"].as_str(), Some("high"));
    }

    #[test]
    fn set_value_with_inline_table() {
        let mut doc = DocumentMut::new();
        let mut inline = toml_edit::InlineTable::new();
        inline.insert("path", toml_edit::Value::from("pathvalue"));
        set_value(
            &mut doc,
            &["harness", "codex", "small_model"],
            toml_edit::Value::InlineTable(inline),
        )
        .unwrap();
        let sm = doc["harness"]["codex"]["small_model"]
            .as_inline_table()
            .unwrap();
        assert_eq!(sm.get("path").and_then(|v| v.as_str()), Some("pathvalue"));
    }

    #[test]
    fn ensure_table_creates_nested() {
        let mut doc = DocumentMut::new();
        let table = ensure_table(&mut doc, &["harness", "codex"]);
        table["small_model"] = toml_edit::Item::Value("anthropic/claude-haiku".into());
        assert_eq!(
            doc["harness"]["codex"]["small_model"].as_str(),
            Some("anthropic/claude-haiku")
        );
    }

    #[test]
    fn round_trip_preserves_formatting() {
        let input = r#"# My agent config
[theta]
schema = "2026-04"

[agent]
name = "test-agent"
description = "A test agent"

[model]
default = "claude-sonnet-4-20250514"
"#;
        let mut doc: DocumentMut = input.parse().unwrap();
        set_value(
            &mut doc,
            &["model", "reasoning_effort"],
            toml_edit::Value::from("high"),
        )
        .unwrap();
        let output = doc.to_string();
        assert!(output.contains("# My agent config"));
        assert!(output.contains("reasoning_effort = \"high\""));
    }
}
