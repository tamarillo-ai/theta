//! JSON/JSONC semantic equality.
//!
//! Parses both files as JSON (stripping JSONC comments), compares as
//! `serde_json::Value` -- key ordering is invisible.

use std::path::Path;

/// Parse a file as JSONC (handles `//` comments and trailing commas).
pub fn parse_jsonc(content: &str) -> Result<serde_json::Value, String> {
    jsonc_parser::parse_to_serde_value(content, &jsonc_parser::ParseOptions::default())
        .map_err(|e| format!("JSONC parse error: {e}"))
}

/// Assert two JSON/JSONC files are semantically equal.
///
/// Strips JSONC comments, parses both, does deep equality on
/// `serde_json::Value`. Key ordering is invisible.
///
/// `ignore_keys`: top-level keys to skip (e.g. theta-added keys like `"type"`).
pub fn assert_json_semantic_equal(original_path: &Path, cast_path: &Path, ignore_keys: &[&str]) {
    let orig_raw = fs_err::read_to_string(original_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original_path.display()));
    let cast_raw = fs_err::read_to_string(cast_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast_path.display()));

    let orig = parse_jsonc(&orig_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", original_path.display()));
    let cast = parse_jsonc(&cast_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", cast_path.display()));

    if ignore_keys.is_empty() {
        if orig != cast {
            let diffs = json_diffs("", &orig, &cast);
            panic!(
                "JSON differs between {} and {}:\n{}",
                original_path.display(),
                cast_path.display(),
                diffs.join("\n"),
            );
        }
    } else {
        // compare with ignored keys stripped
        let orig_filtered = filter_keys(&orig, ignore_keys);
        let cast_filtered = filter_keys(&cast, ignore_keys);
        if orig_filtered != cast_filtered {
            let diffs = json_diffs("", &orig_filtered, &cast_filtered);
            panic!(
                "JSON differs between {} and {} (ignoring {:?}):\n{}",
                original_path.display(),
                cast_path.display(),
                ignore_keys,
                diffs.join("\n"),
            );
        }
    }
}

/// Remove top-level keys from a JSON object.
fn filter_keys(val: &serde_json::Value, ignore: &[&str]) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => {
            let filtered: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter(|(k, _)| !ignore.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            serde_json::Value::Object(filtered)
        }
        other => other.clone(),
    }
}

/// Produce human-readable diffs between two JSON values.
fn json_diffs(path: &str, a: &serde_json::Value, b: &serde_json::Value) -> Vec<String> {
    use serde_json::Value;
    let mut diffs = Vec::new();
    match (a, b) {
        (Value::Object(ma), Value::Object(mb)) => {
            for (k, v) in ma {
                let key_path = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match mb.get(k) {
                    Some(bv) => diffs.extend(json_diffs(&key_path, v, bv)),
                    None => diffs.push(format!("{key_path}: missing in cast")),
                }
            }
            for k in mb.keys() {
                if !ma.contains_key(k) {
                    let key_path = if path.is_empty() {
                        k.clone()
                    } else {
                        format!("{path}.{k}")
                    };
                    diffs.push(format!("{key_path}: added by cast"));
                }
            }
        }
        _ if a != b => {
            let label = if path.is_empty() { "(root)" } else { path };
            diffs.push(format!("{label}: {a} ≠ {b}"));
        }
        _ => {}
    }
    diffs
}
