//! Codex CLI-specific semantic equality checkers.
//!
//! Codex configuration round-trips through:
//! - AGENTS.md (Markdown body, no frontmatter; opaque content)
//! - .codex/config.toml (TOML — key order tolerated; semantic equality only)
//! - .codex/agents/<name>.toml (TOML full-config-layer)
//! - .codex/hooks.json (JSON; key order tolerated)
//! - .agents/skills/ or .codex/skills/ (skill location tolerated)
//!
//! ref: <https://developers.openai.com/codex/config-reference>

use std::path::{Path, PathBuf};

use super::body::assert_body_equal;
use super::json::assert_json_semantic_equal;

/// Assert AGENTS.md round-trips correctly.
///
/// Codex AGENTS.md is opaque developer-instructions. theta does not parse it,
/// so byte-identity modulo whitespace is the contract.
pub fn assert_agents_md_equal(original: &Path, cast: &Path) {
    assert_body_equal(original, cast);
}

/// Assert two TOML files are semantically equal (key order invisible).
///
/// Parses both via the `toml` crate, normalizes to `serde_json::Value`, and
/// deep-compares. String values are compared with trailing whitespace
/// stripped — codex subagent `developer_instructions` round-trips through
/// theta's materialization layer which normalizes the trailing newline (same
/// concession the body checker makes for markdown).
///
/// `ignore_top_level_keys`: top-level keys to remove on both sides before
/// comparing. Use this for synthesized fields like `name` that cast emits
/// regardless of whether the source had them (codex docs mark `name` as
/// required on subagent files; cast fills it in from the filename slug).
pub fn assert_toml_semantic_equal(original: &Path, cast: &Path, ignore_top_level_keys: &[&str]) {
    let orig_raw = fs_err::read_to_string(original)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original.display()));
    let cast_raw = fs_err::read_to_string(cast)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast.display()));

    let orig_val: toml::Value = toml::from_str(&orig_raw)
        .unwrap_or_else(|e| panic!("failed to parse TOML {}: {e}", original.display()));
    let cast_val: toml::Value = toml::from_str(&cast_raw)
        .unwrap_or_else(|e| panic!("failed to parse TOML {}: {e}", cast.display()));

    let mut orig_json = normalize_strings(serde_json::to_value(&orig_val).unwrap());
    let mut cast_json = normalize_strings(serde_json::to_value(&cast_val).unwrap());

    if !ignore_top_level_keys.is_empty() {
        strip_top_level_keys(&mut orig_json, ignore_top_level_keys);
        strip_top_level_keys(&mut cast_json, ignore_top_level_keys);
    }

    assert!(
        orig_json == cast_json,
        "TOML differs between {} and {}\noriginal: {}\ncast:     {}",
        original.display(),
        cast.display(),
        serde_json::to_string_pretty(&orig_json).unwrap_or_default(),
        serde_json::to_string_pretty(&cast_json).unwrap_or_default(),
    );
}

/// Convenience wrapper for `.codex/config.toml`.
///
/// Ignores top-level `hooks`: this checker compares config.toml in isolation,
/// but hooks have a special round-trip property because codex accepts them
/// in two forms (`hooks.json` and inline `[hooks]`) and MERGES them at
/// startup. Theta cast always emits the JSON file form (decision D2). So a
/// fixture with inline `[hooks]` correctly round-trips to `config.toml`
/// without `[hooks]` plus a new `.codex/hooks.json` — these are
/// semantically equivalent per the codex docs.
///
/// Use `assert_codex_hooks_union_equal(project_dir_orig, project_dir_cast)`
/// alongside this check to verify the hooks UNION across both files matches.
///
/// ref: <https://developers.openai.com/codex/hooks#where-codex-looks-for-hooks>
/// ref: notes.rs `CAST_HOOKS_JSON`
pub fn assert_config_toml_equal(original: &Path, cast: &Path) {
    assert_toml_semantic_equal(original, cast, &["hooks"]);
}

/// Assert that the codex hooks UNION across `.codex/config.toml` `[hooks]`
/// and `.codex/hooks.json` is semantically equal between two project trees.
///
/// Codex merges the two surfaces at startup; theta cast normalizes to the
/// JSON file form. So a fixture with inline `[hooks]` round-trips to a
/// `.codex/hooks.json` containing the same hook tree (modulo TOML/JSON value
/// representation). This checker verifies that property.
///
/// `original_root` and `cast_root` are the project root directories. The
/// checker reads `.codex/config.toml` and `.codex/hooks.json` from each side
/// and computes the merged hook map for both.
///
/// ref: <https://developers.openai.com/codex/hooks#where-codex-looks-for-hooks>
pub fn assert_codex_hooks_union_equal(original_root: &Path, cast_root: &Path) {
    let orig = read_hooks_union(original_root);
    let cast = read_hooks_union(cast_root);

    let orig_norm = normalize_strings(orig.clone());
    let cast_norm = normalize_strings(cast.clone());

    assert!(
        orig_norm == cast_norm,
        "codex hooks UNION (config.toml [hooks] + hooks.json) differs between projects\n  original root: {}\n  cast root:     {}\noriginal hooks: {}\ncast hooks:     {}",
        original_root.display(),
        cast_root.display(),
        serde_json::to_string_pretty(&orig_norm).unwrap_or_default(),
        serde_json::to_string_pretty(&cast_norm).unwrap_or_default(),
    );
}

/// Read the merged hooks payload from a codex project root.
///
/// Returns the merged JSON value of:
/// - `<root>/.codex/config.toml`'s top-level `hooks` table (if present), and
/// - `<root>/.codex/hooks.json`'s top-level `hooks` field (if present).
///
/// Per the codex docs, when both exist codex merges them. Merging here means:
/// for each event name (e.g. `PreToolUse`), concatenate the arrays from both
/// sources. This mirrors codex's "loads all matching hooks" behavior.
///
/// Returns `Null` when neither source has hooks.
fn read_hooks_union(project_root: &Path) -> serde_json::Value {
    use serde_json::{Map as JsonMap, Value as JsonValue};

    let config_hooks = read_config_toml_hooks(project_root).unwrap_or_default();
    let json_hooks = read_hooks_json(project_root).unwrap_or_default();

    if config_hooks.is_empty() && json_hooks.is_empty() {
        return JsonValue::Null;
    }

    // merge: for each event key, concatenate matcher-group arrays.
    let mut merged: JsonMap<String, JsonValue> = JsonMap::new();
    for source in [&config_hooks, &json_hooks] {
        for (event, value) in source {
            let entry = merged
                .entry(event.clone())
                .or_insert_with(|| JsonValue::Array(Vec::new()));
            let arr = entry
                .as_array_mut()
                .expect("event value must be array (we just inserted one)");
            match value {
                JsonValue::Array(items) => arr.extend(items.iter().cloned()),
                // non-array event values (unusual but tolerated): push as-is
                other => arr.push(other.clone()),
            }
        }
    }
    JsonValue::Object(merged)
}

/// Read `[hooks]` from `.codex/config.toml` as a JSON map. None if the file
/// is absent or has no `hooks` key.
fn read_config_toml_hooks(
    project_root: &Path,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    let path = project_root.join(".codex/config.toml");
    if !path.is_file() {
        return None;
    }
    let raw = fs_err::read_to_string(&path).ok()?;
    let toml_val: toml::Value = toml::from_str(&raw).ok()?;
    let json_val: serde_json::Value = serde_json::to_value(&toml_val).ok()?;
    json_val.get("hooks")?.as_object().cloned()
}

/// Read `hooks` from `.codex/hooks.json` as a JSON map. Accepts both shapes:
/// the file may be `{"hooks": {...}}` or just `{...}` directly. None if the
/// file is absent or malformed.
fn read_hooks_json(project_root: &Path) -> Option<serde_json::Map<String, serde_json::Value>> {
    let path = project_root.join(".codex/hooks.json");
    if !path.is_file() {
        return None;
    }
    let raw = fs_err::read_to_string(&path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&raw).ok()?;
    if let Some(inner) = val.get("hooks").and_then(|v| v.as_object()) {
        return Some(inner.clone());
    }
    val.as_object().cloned()
}

/// Convenience wrapper for `.codex/agents/<name>.toml`.
///
/// Subagent `name` policy: the codex docs mark `name` as REQUIRED on subagent
/// files (`developers.openai.com/codex/subagents#custom-agent-file-schema`).
/// In practice many real-world repos violate the spec and omit it (codex
/// falls back to the filename slug). Theta cast emits `name` unconditionally
/// to produce spec-compliant output.
///
/// The checker enforces the strongest contract that survives this asymmetry:
/// - If the original had `name`: cast's `name` MUST equal it (semantic equality).
/// - If the original lacked `name`: cast's `name` MUST equal the filename
///   stem (verifies cast synthesizes the correct value, doesn't invent
///   something random).
///
/// After that name check, the full TOML compare runs with `name` removed from
/// both sides so the synthesis isn't double-counted.
pub fn assert_subagent_toml_equal(original: &Path, cast: &Path) {
    let orig_val: toml::Value = parse_toml(original);
    let cast_val: toml::Value = parse_toml(cast);

    let orig_name = orig_val.get("name").and_then(|v| v.as_str());
    let cast_name = cast_val.get("name").and_then(|v| v.as_str());
    let filename_stem = cast
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("subagent file must have a UTF-8 stem");

    match (orig_name, cast_name) {
        (Some(o), Some(c)) => {
            assert_eq!(
                o,
                c,
                "subagent `name` differs between original and cast\n  original: {}\n  cast:     {}",
                original.display(),
                cast.display()
            );
        }
        (None, Some(c)) => {
            assert_eq!(
                c,
                filename_stem,
                "cast synthesized `name = {c:?}` but the filename stem is {filename_stem:?}\n  cast: {}",
                cast.display()
            );
        }
        (None, None) => {
            // both omitted; codex docs say required but in practice this is
            // tolerated. nothing to verify here.
        }
        (Some(_), None) => {
            panic!(
                "subagent original has `name` but cast dropped it\n  original: {}\n  cast:     {}",
                original.display(),
                cast.display()
            );
        }
    }

    // now compare the rest of the document with `name` removed from both sides.
    assert_toml_semantic_equal(original, cast, &["name"]);
}

fn parse_toml(path: &Path) -> toml::Value {
    let raw = fs_err::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    toml::from_str(&raw).unwrap_or_else(|e| panic!("failed to parse TOML {}: {e}", path.display()))
}

fn strip_top_level_keys(value: &mut serde_json::Value, keys: &[&str]) {
    if let serde_json::Value::Object(map) = value {
        for k in keys {
            map.remove(*k);
        }
    }
}

/// Recursively strip trailing whitespace from every string in a JSON tree.
/// Mirrors the trailing-newline tolerance that `body::assert_body_equal`
/// applies to Markdown body content - applied here for TOML string fields
/// like `developer_instructions`.
fn normalize_strings(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::String(s) => serde_json::Value::String(s.trim_end().to_string()),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(normalize_strings).collect())
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, normalize_strings(v)))
                .collect(),
        ),
        other => other,
    }
}

/// Assert `.codex/hooks.json` round-trips semantically.
///
/// Strips JSONC comments, compares as JSON objects. Key ordering invisible.
pub fn assert_hooks_equal(original: &Path, cast: &Path) {
    assert_json_semantic_equal(original, cast, &[]);
}

/// Assert that a skill round-trips. Tolerates relocation between
/// `.agents/skills/<name>/` and `.codex/skills/<name>/` by trying both paths
/// on the cast side.
///
/// `original_rel` is the path relative to the project root in the original
/// fixture (e.g. `.agents/skills/lint-check/SKILL.md`).
pub fn assert_skill_equal_relocatable(original_rel: &str, original_root: &Path, cast_root: &Path) {
    let cast_candidates = relocated_skill_paths(original_rel);

    let cast_path = cast_candidates
        .iter()
        .find(|p| cast_root.join(p).exists())
        .unwrap_or_else(|| {
            panic!("skill {original_rel} not found at any relocated path: {cast_candidates:?}")
        });

    let original = original_root.join(original_rel);
    let cast = cast_root.join(cast_path);
    assert_body_equal(&original, &cast);
}

/// Given an original skill path under `.agents/skills/` or `.codex/skills/`,
/// return the candidate locations the cast may have written it to.
fn relocated_skill_paths(original_rel: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    out.push(PathBuf::from(original_rel));
    if let Some(tail) = original_rel.strip_prefix(".agents/skills/") {
        out.push(PathBuf::from(format!(".codex/skills/{tail}")));
    } else if let Some(tail) = original_rel.strip_prefix(".codex/skills/") {
        out.push(PathBuf::from(format!(".agents/skills/{tail}")));
    }
    out
}
