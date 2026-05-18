//! Claude Code-specific semantic equality checkers.
//!
//! Claude rules use `paths` for globs (not `applyTo`). Tools in agent
//! frontmatter may be comma-separated strings or YAML sequences. Key ordering
//! and YAML quote style differences are tolerated.
//!
//! ref: <https://code.claude.com/docs/en/memory>
//! ref: <https://code.claude.com/docs/en/sub-agents>
//! ref: <https://code.claude.com/docs/en/mcp>

use std::path::Path;

use super::body::assert_body_equal;
use super::frontmatter::{assert_frontmatter_equal, parse_document};
use super::json::assert_json_semantic_equal;

/// Assert a Claude rule (`.claude/rules/*.md`) round-trips correctly.
///
/// Checks frontmatter `paths` (as sorted patterns) and body content.
pub fn assert_rule_equal(original: &Path, cast: &Path) {
    assert_frontmatter_equal(original, cast, &[]);
    assert_body_equal(original, cast);
}

/// Assert a Claude agent (`.claude/agents/*.md`) round-trips correctly.
///
/// Tolerates: key reordering, `tools` comma-string vs block list, `name` added.
/// Ignores `name` in the comparison since cast always emits it.
pub fn assert_agent_equal(original: &Path, cast: &Path) {
    let orig_raw = fs_err::read_to_string(original)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original.display()));
    let cast_raw = fs_err::read_to_string(cast)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast.display()));

    let orig_doc = parse_document(&orig_raw);
    let cast_doc = parse_document(&cast_raw);

    // normalize tools: comma-string to sorted array for both sides
    let mut orig_fm = orig_doc.frontmatter;
    let mut cast_fm = cast_doc.frontmatter;

    normalize_tools(&mut orig_fm);
    normalize_tools(&mut cast_fm);

    // compare all frontmatter keys except `name` (cast always adds it)
    for (k, v) in &orig_fm {
        if k == "name" {
            continue;
        }
        let cast_v = cast_fm.get(k).unwrap_or_else(|| {
            panic!(
                "frontmatter key `{k}` present in original but missing in cast\n  original: {}\n  cast: {}",
                original.display(),
                cast.display()
            )
        });
        assert_eq!(
            v,
            cast_v,
            "frontmatter key `{k}` differs\n  original: {}\n  cast: {}",
            original.display(),
            cast.display()
        );
    }

    // body
    assert_body_equal(original, cast);
}

/// Assert a Claude `settings.json` round-trips semantically.
///
/// Strips JSONC comments, compares as JSON objects. Key ordering invisible.
pub fn assert_settings_equal(original: &Path, cast: &Path) {
    assert_json_semantic_equal(original, cast, &[]);
}

/// Assert a Claude `.mcp.json` round-trips semantically.
///
/// Tolerates theta adding `"type": "stdio"` or `"type": "http"` to server
/// configs, and key reordering within server objects.
pub fn assert_mcp_equal(original: &Path, cast: &Path) {
    let orig_raw = fs_err::read_to_string(original)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original.display()));
    let cast_raw = fs_err::read_to_string(cast)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast.display()));

    let orig: serde_json::Value = super::json::parse_jsonc(&orig_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", original.display()));
    let cast_val: serde_json::Value = super::json::parse_jsonc(&cast_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", cast.display()));

    let orig_servers = orig
        .get(theta_harness::layout::ClaudeCodeLayout::MCP_ROOT_KEY)
        .and_then(|s| s.as_object());
    let cast_servers = cast_val
        .get(theta_harness::layout::ClaudeCodeLayout::MCP_ROOT_KEY)
        .and_then(|s| s.as_object());

    match (orig_servers, cast_servers) {
        (Some(os), Some(cs)) => {
            for (name, orig_server) in os {
                let cast_server = cs
                    .get(name)
                    .unwrap_or_else(|| panic!("mcp server \"{name}\" missing from cast output"));
                if let (Some(om), Some(cm)) = (orig_server.as_object(), cast_server.as_object()) {
                    for (k, v) in om {
                        if k == "type" {
                            continue;
                        }
                        let cv = cm.get(k).unwrap_or_else(|| {
                            panic!("mcp server \"{name}\" missing key \"{k}\" in cast output")
                        });
                        assert_eq!(
                            v, cv,
                            "mcp server \"{name}\" key \"{k}\" differs\n  original: {v}\n  cast: {cv}"
                        );
                    }
                }
            }
        }
        (None, None) => {}
        _ => panic!(
            "mcpServers mismatch: original has servers={}, cast has servers={}",
            orig_servers.is_some(),
            cast_servers.is_some()
        ),
    }
}

/// Normalize `tools` from comma-separated string to sorted array.
fn normalize_tools(fm: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(tools) = fm.get("tools") {
        let normalized = match tools {
            serde_json::Value::String(s) => {
                let mut items: Vec<String> = s.split(',').map(|t| t.trim().to_string()).collect();
                items.sort();
                serde_json::Value::Array(items.into_iter().map(serde_json::Value::String).collect())
            }
            serde_json::Value::Array(arr) => {
                let mut items: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                    .collect();
                items.sort();
                serde_json::Value::Array(items.into_iter().map(serde_json::Value::String).collect())
            }
            other => other.clone(),
        };
        fm.insert("tools".to_string(), normalized);
    }
}
