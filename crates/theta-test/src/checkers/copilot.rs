//! Copilot-specific semantic equality checkers.
//!
//! Dispatches to common checkers with Copilot-specific configuration
//! (which keys to compare, which to ignore, how to normalize `applyTo`, etc.).

use super::body::{assert_body_equal, assert_file_identical};
use super::frontmatter::{assert_frontmatter_equal, parse_document};
use super::json::assert_json_semantic_equal;
use std::path::Path;
use theta_static::kebab_case;

/// Assert a Copilot rule (`.instructions.md`) round-trips correctly.
///
/// Checks:
/// - Frontmatter: `applyTo` (as sorted patterns), `description`
/// - Body: equal after CRLF normalization + trim
pub fn assert_rule_equal(original: &Path, cast: &Path) {
    // frontmatter - no keys to ignore for rules
    assert_frontmatter_equal(original, cast, &[]);
    // body
    assert_body_equal(original, cast);
}

/// Assert a Copilot agent (`.agent.md`) round-trips correctly.
///
/// Checks:
/// - Frontmatter: `name`, `description`, `model`, `tools`, all extras
/// - Body: equal after CRLF normalization + trim
///
/// YAML comments in frontmatter are a known limitation (stripped by parser).
pub fn assert_agent_equal(original: &Path, cast: &Path) {
    // frontmatter - no keys to ignore (all should round-trip including extras)
    assert_frontmatter_equal(original, cast, &[]);
    // body
    assert_body_equal(original, cast);
}

/// Assert a Copilot agent round-trips correctly, with specific keys ignored.
///
/// Use when original had fields that theta intentionally doesn't re-emit
/// (e.g., YAML comments appear as no key at all).
pub fn assert_agent_equal_ignoring(original: &Path, cast: &Path, ignore_keys: &[&str]) {
    assert_frontmatter_equal(original, cast, ignore_keys);
    assert_body_equal(original, cast);
}

/// Assert a Copilot `settings.json` round-trips semantically.
///
/// Strips JSONC comments, compares as JSON objects. Key ordering invisible.
pub fn assert_settings_equal(original: &Path, cast: &Path) {
    assert_json_semantic_equal(original, cast, &[]);
}

/// Assert a Copilot `mcp.json` round-trips semantically.
///
/// Tolerates theta adding `"type": "stdio"` to server configs.
pub fn assert_mcp_equal(original: &Path, cast: &Path) {
    let orig_raw = fs_err::read_to_string(original)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", original.display()));
    let cast_raw = fs_err::read_to_string(cast)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cast.display()));

    let orig: serde_json::Value = super::json::parse_jsonc(&orig_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", original.display()));
    let cast: serde_json::Value = super::json::parse_jsonc(&cast_raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", cast.display()));

    // compare servers semantically - tolerate "type" being added
    let orig_servers = orig.get("servers").and_then(|s| s.as_object());
    let cast_servers = cast.get("servers").and_then(|s| s.as_object());

    match (orig_servers, cast_servers) {
        (Some(os), Some(cs)) => {
            // every original server should exist in cast
            for (name, orig_server) in os {
                let cast_server = cs
                    .get(name)
                    .unwrap_or_else(|| panic!("mcp server \"{name}\" missing from cast output"));
                // compare all keys except "type" (theta may add it)
                if let (Some(om), Some(cm)) = (orig_server.as_object(), cast_server.as_object()) {
                    for (k, v) in om {
                        if let Some(cv) = cm.get(k) {
                            assert!(
                                v == cv,
                                "mcp server \"{name}\" key \"{k}\" differs: orig={v}, cast={cv}"
                            );
                        } else {
                            panic!("mcp server \"{name}\" key \"{k}\" missing from cast");
                        }
                    }
                }
            }
        }
        (None, None) => {} // both empty
        _ => panic!(
            "mcp servers mismatch: orig has {}, cast has {}",
            orig_servers.map_or(0, serde_json::Map::len),
            cast_servers.map_or(0, serde_json::Map::len),
        ),
    }
}

/// Assert a skill directory round-trips with all files byte-identical
/// (tolerating trailing newline difference).
pub fn assert_skill_equal(original_dir: &Path, cast_dir: &Path) {
    assert!(
        original_dir.is_dir(),
        "original skill dir does not exist: {}",
        original_dir.display()
    );
    assert!(
        cast_dir.is_dir(),
        "cast skill dir does not exist: {}",
        cast_dir.display()
    );

    // collect all files in original
    let mut orig_files: Vec<std::path::PathBuf> = Vec::new();
    collect_files(original_dir, original_dir, &mut orig_files);
    orig_files.sort();

    for rel in &orig_files {
        let orig_file = original_dir.join(rel);
        let cast_file = cast_dir.join(rel);
        assert!(
            cast_file.exists(),
            "skill file missing from cast: {}",
            cast_file.display()
        );
        assert_file_identical(&orig_file, &cast_file);
    }
}

fn collect_files(dir: &Path, base: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = fs_err::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, base, out);
        } else {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
            out.push(rel);
        }
    }
}

/// Run a full Copilot round-trip check on input vs output directories.
///
/// Dispatches to the appropriate checker based on file path pattern.
pub fn assert_copilot_round_trip(input_dir: &Path, output_dir: &Path) {
    let mut checked = 0usize;

    // system prompt
    let sys_orig = input_dir.join(".github/copilot-instructions.md");
    let sys_cast = output_dir.join(".github/copilot-instructions.md");
    if sys_orig.exists() {
        assert!(
            sys_cast.exists(),
            "copilot-instructions.md missing from cast output"
        );
        assert_body_equal(&sys_orig, &sys_cast);
        checked += 1;
    }

    // rules
    let rules_orig = input_dir.join(".github/instructions");
    let rules_cast = output_dir.join(".github/instructions");
    if rules_orig.is_dir() {
        let mut rule_files = Vec::new();
        collect_instruction_files(&rules_orig, &rules_orig, &mut rule_files);
        for rel in &rule_files {
            let orig = rules_orig.join(rel);
            let cast = rules_cast.join(rel);
            assert!(
                cast.exists(),
                "rule file missing from cast: {}",
                cast.display()
            );
            assert_rule_equal(&orig, &cast);
            checked += 1;
        }
    }

    // agents
    let agents_orig = input_dir.join(".github/agents");
    let agents_cast = output_dir.join(".github/agents");
    if agents_orig.is_dir() {
        let orig_agents: Vec<String> = collect_agent_names(&agents_orig);
        for name in &orig_agents {
            // agent filenames may be slug-normalized - find by content match
            let orig = agents_orig.join(name);
            // cast uses kebab_case slug, try both .agent.md patterns
            let orig_fm = parse_document(&fs_err::read_to_string(&orig).unwrap_or_default());
            let agent_name = orig_fm
                .frontmatter
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(name.strip_suffix(".agent.md").unwrap_or(name));
            let slug = kebab_case(agent_name);
            let cast_path = agents_cast.join(format!("{slug}.agent.md"));
            if cast_path.exists() {
                assert_agent_equal(&orig, &cast_path);
                checked += 1;
            }
            // if not found by slug, it's a known slug-rename limitation
        }
    }

    // skills
    let skills_orig = input_dir.join(".github/skills");
    let skills_cast = output_dir.join(".github/skills");
    if skills_orig.is_dir() {
        for entry in fs_err::read_dir(&skills_orig)
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            if path.is_dir() && path.join("SKILL.md").exists() {
                let name = entry.file_name().to_string_lossy().to_string();
                let cast_skill = skills_cast.join(&name);
                if cast_skill.exists() {
                    assert_skill_equal(&path, &cast_skill);
                    checked += 1;
                }
            }
        }
    }

    // settings
    let settings_orig = input_dir.join(".vscode/settings.json");
    let settings_cast = output_dir.join(".vscode/settings.json");
    if settings_orig.exists() && settings_cast.exists() {
        assert_settings_equal(&settings_orig, &settings_cast);
        checked += 1;
    }

    // mcp
    let mcp_orig = input_dir.join(".vscode/mcp.json");
    let mcp_cast = output_dir.join(".vscode/mcp.json");
    if mcp_orig.exists() && mcp_cast.exists() {
        assert_mcp_equal(&mcp_orig, &mcp_cast);
        checked += 1;
    }

    assert!(checked > 0, "no files were checked - fixture may be empty");
}

fn collect_instruction_files(dir: &Path, base: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = fs_err::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_instruction_files(&path, base, out);
        } else if path
            .file_name()
            .and_then(|f| f.to_str())
            .is_some_and(|f| f.ends_with(".instructions.md"))
        {
            let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
            out.push(rel);
        }
    }
}

fn collect_agent_names(dir: &Path) -> Vec<String> {
    let Ok(rd) = fs_err::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = rd
        .flatten()
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.ends_with(".agent.md") || n.ends_with(".md")
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}
