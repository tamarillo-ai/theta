//! Cursor-specific semantic equality checkers.
//!
//! Handles Cursor's frontmatter quirks:
//!
//! - `description: ` (YAML null) normalizes to absent
//! - `globs: *.ts` has leading `*` (invalid YAML c-alias) — pre-quoted before parse
//! - Comma-separated globs normalize to YAML block lists
//! - Key ordering is not preserved
//!
//! ref: <https://cursor.com/docs/rules>
//! ref: <https://cursor.com/docs/subagents>
//! ref: <https://cursor.com/docs/mcp>

use std::path::Path;

use super::body::assert_body_equal;
use super::frontmatter::{frontmatter_diffs, parse_document};
use super::json;

/// Pre-quote frontmatter values starting with `*` so YAML parser doesn't choke.
/// ref: <https://yaml.org/spec/1.2.2>/ §5.3 - [14] c-alias ::= '*'
fn quote_leading_stars(raw: &str) -> String {
    raw.lines()
        .map(|line| {
            if let Some((key, val)) = line.split_once(':') {
                let trimmed = val.trim_start();
                if trimmed.starts_with('*') {
                    return format!("{}: \"{}\"", key.trim_end(), trimmed);
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
/// Normalize globs to a sorted array of strings.
/// Cursor uses scalar `globs: *.ts` or comma-separated `globs: *.ts, *.tsx`,
/// theta emits YAML block list `globs:\n- '*.ts'`. Both are equivalent.
fn normalize_globs(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::String(s) => {
            let mut patterns: Vec<String> = s.split(',').map(|p| p.trim().to_string()).collect();
            patterns.sort();
            serde_json::Value::Array(
                patterns
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            )
        }
        serde_json::Value::Array(arr) => {
            let mut patterns: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                .collect();
            patterns.sort();
            serde_json::Value::Array(
                patterns
                    .into_iter()
                    .map(serde_json::Value::String)
                    .collect(),
            )
        }
        other => other.clone(),
    }
}
/// Assert a cursor rule (`.mdc`) round-trips semantically.
///
/// Tolerates: empty field stripping, key reorder, comma --> list, trailing newline.
pub fn assert_rule_equal(original: &Path, cast: &Path) {
    let orig_raw = fs_err::read_to_string(original)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original.display()));
    let cast_raw = fs_err::read_to_string(cast)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast.display()));

    let orig = parse_document(&quote_leading_stars(&orig_raw));
    let cast_doc = parse_document(&quote_leading_stars(&cast_raw));

    // normalize for known cosmetic differences:
    // - YAML null → absent (empty description/globs)
    // - scalar glob → single-element array (Cursor uses string, theta emits array)
    let orig_fm: serde_json::Map<String, serde_json::Value> = orig
        .frontmatter
        .into_iter()
        .filter(|(_, v)| !v.is_null())
        .map(|(k, v)| {
            if k == "globs" {
                (k, normalize_globs(&v))
            } else {
                (k, v)
            }
        })
        .collect();

    let cast_fm: serde_json::Map<String, serde_json::Value> = cast_doc
        .frontmatter
        .into_iter()
        .map(|(k, v)| {
            if k == "globs" {
                (k, normalize_globs(&v))
            } else {
                (k, v)
            }
        })
        .collect();

    let diffs = frontmatter_diffs(&orig_fm, &cast_fm, &[]);
    assert!(
        diffs.is_empty(),
        "cursor rule frontmatter differs between {} and {}:\n{}",
        original.display(),
        cast.display(),
        diffs.join("\n"),
    );

    // body check - use star-quoted parse to correctly split frontmatter from body
    let orig_body = parse_document(&quote_leading_stars(&orig_raw)).body;
    let cast_body = parse_document(&quote_leading_stars(&cast_raw)).body;
    let orig_norm = orig_body.replace("\r\n", "\n").trim().to_string();
    let cast_norm = cast_body.replace("\r\n", "\n").trim().to_string();
    assert!(
        orig_norm == cast_norm,
        "cursor rule body differs between {} and {}:\norig: {:?}\ncast: {:?}",
        original.display(),
        cast.display(),
        orig_norm.lines().take(5).collect::<Vec<_>>(),
        cast_norm.lines().take(5).collect::<Vec<_>>(),
    )
}

/// Assert a cursor agent (`.md`) round-trips semantically.
///
/// `name` is derived from filename — not expected to round-trip.
pub fn assert_agent_equal(original: &Path, cast: &Path) {
    assert_agent_equal_ignoring(original, cast, &["name"]);
}

/// Assert a cursor agent round-trips, ignoring specified keys.
pub fn assert_agent_equal_ignoring(original: &Path, cast: &Path, ignore_keys: &[&str]) {
    let orig_raw = fs_err::read_to_string(original)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original.display()));
    let cast_raw = fs_err::read_to_string(cast)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast.display()));

    let orig = parse_document(&orig_raw);
    let cast_doc = parse_document(&cast_raw);

    let diffs = frontmatter_diffs(&orig.frontmatter, &cast_doc.frontmatter, ignore_keys);
    assert!(
        diffs.is_empty(),
        "cursor agent frontmatter differs between {} and {}:\n{}",
        original.display(),
        cast.display(),
        diffs.join("\n"),
    );

    assert_body_equal(original, cast);
}

/// Assert cursor `mcp.json` round-trips semantically.
///
/// Tolerates: key reorder, pretty-print, unmodeled fields missing from cast.
pub fn assert_mcp_equal(original: &Path, cast: &Path) {
    let orig_raw = fs_err::read_to_string(original)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original.display()));
    let cast_raw = fs_err::read_to_string(cast)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast.display()));

    let orig: serde_json::Value = json::parse_jsonc(&orig_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", original.display()));
    let cast_val: serde_json::Value = json::parse_jsonc(&cast_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", cast.display()));

    let orig_servers = orig.get("mcpServers").and_then(|s| s.as_object());
    let cast_servers = cast_val.get("mcpServers").and_then(|s| s.as_object());

    match (orig_servers, cast_servers) {
        (Some(os), Some(cs)) => {
            for (name, orig_server) in os {
                let cast_server = cs
                    .get(name)
                    .unwrap_or_else(|| panic!("mcp server \"{name}\" missing from cast"));
                if let (Some(om), Some(cm)) = (orig_server.as_object(), cast_server.as_object()) {
                    for (k, v) in om {
                        if let Some(cv) = cm.get(k) {
                            assert!(v == cv, "mcp server \"{name}\".{k}: orig={v}, cast={cv}");
                        }
                    }
                }
            }
        }
        (None, None) => {}
        _ => panic!("mcpServers mismatch"),
    }
}

/// Assert a cursor skill directory round-trips (`SKILL.md` + subdirs).
pub fn assert_skill_equal(original: &Path, cast: &Path) {
    let orig_skill = original.join("SKILL.md");
    let cast_skill = cast.join("SKILL.md");
    if orig_skill.exists() {
        assert!(
            cast_skill.exists(),
            "SKILL.md missing from cast: {}",
            cast.display()
        );
        assert_body_equal(&orig_skill, &cast_skill);
    }
}
