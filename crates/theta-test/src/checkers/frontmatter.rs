//! YAML frontmatter semantic equality.
//!
//! Parses `---\n...\n---` frontmatter from both files, compares as
//! `serde_json::Value` maps. Quote style, key ordering, inline vs block
//! arrays are invisible after parsing.

use std::path::Path;

/// Parsed frontmatter + body from a markdown file with optional YAML frontmatter.
pub struct ParsedDocument {
    /// Frontmatter fields as a JSON map (empty if no frontmatter)
    pub frontmatter: serde_json::Map<String, serde_json::Value>,
    /// Body content after the closing `---`
    pub body: String,
}

/// Split `---\nyaml\n---\nbody` into (`yaml_str`, body).
fn split_frontmatter(input: &str) -> (Option<&str>, &str) {
    let trimmed = input.strip_prefix("---").unwrap_or(input);
    if std::ptr::eq(trimmed, input) {
        return (None, input);
    }
    let Some(end) = trimmed.find("\n---") else {
        return (None, input);
    };
    let yaml = trimmed[..end].trim();
    let after = &trimmed[end + 4..];
    let body = after.strip_prefix('\n').unwrap_or(after);
    if yaml.is_empty() {
        (None, body)
    } else {
        (Some(yaml), body)
    }
}

/// Parse a markdown file with optional YAML frontmatter.
pub fn parse_document(content: &str) -> ParsedDocument {
    let (yaml_str, body) = split_frontmatter(content);
    let Some(yaml_str) = yaml_str else {
        return ParsedDocument {
            frontmatter: serde_json::Map::new(),
            body: body.to_string(),
        };
    };
    match serde_norway::from_str::<serde_json::Value>(yaml_str) {
        Ok(serde_json::Value::Object(map)) => ParsedDocument {
            frontmatter: map,
            body: body.to_string(),
        },
        _ => ParsedDocument {
            frontmatter: serde_json::Map::new(),
            body: content.to_string(),
        },
    }
}

/// Assert that two markdown files have semantically equal frontmatter.
///
/// Compares frontmatter fields as JSON values after parsing. Key order,
/// quote style, and inline vs block arrays are invisible.
///
/// `ignore_keys`: fields to skip during comparison (e.g., harness-specific
/// fields that are known to not round-trip).
///
/// Returns a list of differences (empty = equal).
pub fn frontmatter_diffs(
    original: &serde_json::Map<String, serde_json::Value>,
    cast: &serde_json::Map<String, serde_json::Value>,
    ignore_keys: &[&str],
) -> Vec<String> {
    let mut diffs = Vec::new();

    // check keys in original that are missing or different in cast
    for (k, v) in original {
        if ignore_keys.contains(&k.as_str()) {
            continue;
        }
        match cast.get(k) {
            Some(cv) => {
                if !values_semantically_equal(v, cv) {
                    diffs.push(format!("frontmatter key \"{k}\": original={v}, cast={cv}"));
                }
            }
            None => {
                diffs.push(format!(
                    "frontmatter key \"{k}\" present in original, missing in cast"
                ));
            }
        }
    }

    // check keys in cast that are not in original (unexpected additions)
    for k in cast.keys() {
        if ignore_keys.contains(&k.as_str()) {
            continue;
        }
        if !original.contains_key(k) {
            diffs.push(format!(
                "frontmatter key \"{k}\" added by cast (not in original)"
            ));
        }
    }

    diffs
}

/// Compare two JSON values semantically.
///
/// Arrays are compared as sorted sets of strings for simple string arrays
/// (like `tools`). Objects are compared recursively. Everything else uses
/// standard equality.
fn values_semantically_equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value;
    match (a, b) {
        (Value::Array(va), Value::Array(vb)) => {
            // for arrays of strings, compare as sorted sets
            let all_strings_a = va.iter().all(serde_json::Value::is_string);
            let all_strings_b = vb.iter().all(serde_json::Value::is_string);
            if all_strings_a && all_strings_b {
                let mut sa: Vec<&str> = va.iter().filter_map(|v| v.as_str()).collect();
                let mut sb: Vec<&str> = vb.iter().filter_map(|v| v.as_str()).collect();
                sa.sort_unstable();
                sb.sort_unstable();
                sa == sb
            } else {
                // for mixed arrays, compare element by element
                va.len() == vb.len()
                    && va
                        .iter()
                        .zip(vb.iter())
                        .all(|(x, y)| values_semantically_equal(x, y))
            }
        }
        (Value::Object(ma), Value::Object(mb)) => {
            if ma.len() != mb.len() {
                return false;
            }
            ma.iter()
                .all(|(k, v)| mb.get(k).is_some_and(|bv| values_semantically_equal(v, bv)))
        }
        // strings that look like comma-separated patterns (e.g. `applyTo`):
        // normalize by splitting on `,`, trim each, compare as sorted sets
        (Value::String(sa), Value::String(sb)) => {
            if sa.contains(',') || sb.contains(',') {
                let mut pa: Vec<&str> = sa.split(',').map(str::trim).collect();
                let mut pb: Vec<&str> = sb.split(',').map(str::trim).collect();
                pa.sort_unstable();
                pb.sort_unstable();
                pa == pb
            } else {
                sa == sb
            }
        }
        _ => a == b,
    }
}

/// Assert frontmatter equality. Panics with a detailed message on failure.
pub fn assert_frontmatter_equal(original_path: &Path, cast_path: &Path, ignore_keys: &[&str]) {
    let orig_content = fs_err::read_to_string(original_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original_path.display()));
    let cast_content = fs_err::read_to_string(cast_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast_path.display()));

    let orig = parse_document(&orig_content);
    let cast = parse_document(&cast_content);

    let diffs = frontmatter_diffs(&orig.frontmatter, &cast.frontmatter, ignore_keys);
    assert!(
        diffs.is_empty(),
        "frontmatter differs between {} and {}:\n{}",
        original_path.display(),
        cast_path.display(),
        diffs.join("\n"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_style_is_invisible() {
        let orig = parse_document("---\napplyTo: \"**/*.rs\"\n---\nbody");
        let cast = parse_document("---\napplyTo: '**/*.rs'\n---\nbody");
        let diffs = frontmatter_diffs(&orig.frontmatter, &cast.frontmatter, &[]);
        assert!(diffs.is_empty(), "diffs: {diffs:?}");
    }

    #[test]
    fn key_order_is_invisible() {
        let orig = parse_document("---\ndescription: foo\napplyTo: bar\n---\n");
        let cast = parse_document("---\napplyTo: bar\ndescription: foo\n---\n");
        let diffs = frontmatter_diffs(&orig.frontmatter, &cast.frontmatter, &[]);
        assert!(diffs.is_empty(), "diffs: {diffs:?}");
    }

    #[test]
    fn inline_vs_block_array_is_invisible() {
        let orig = parse_document("---\ntools: ['read', 'search']\n---\n");
        let cast = parse_document("---\ntools:\n- read\n- search\n---\n");
        let diffs = frontmatter_diffs(&orig.frontmatter, &cast.frontmatter, &[]);
        assert!(diffs.is_empty(), "diffs: {diffs:?}");
    }

    #[test]
    fn missing_key_is_detected() {
        let orig = parse_document("---\napplyTo: \"**\"\ndescription: hi\n---\n");
        let cast = parse_document("---\napplyTo: \"**\"\n---\n");
        let diffs = frontmatter_diffs(&orig.frontmatter, &cast.frontmatter, &[]);
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].contains("description"));
    }

    #[test]
    fn ignored_key_is_skipped() {
        let orig = parse_document("---\napplyTo: \"**\"\nextra: gone\n---\n");
        let cast = parse_document("---\napplyTo: \"**\"\n---\n");
        let diffs = frontmatter_diffs(&orig.frontmatter, &cast.frontmatter, &["extra"]);
        assert!(diffs.is_empty());
    }
}
