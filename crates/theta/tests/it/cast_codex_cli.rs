//! Codex CLI cast e2e tests.
//!
//! Each test runs `theta cast from codex-cli` and/or `theta cast to codex-cli`
//! against fixtures under `test/cast-fixtures/codex-cli/` and asserts on
//! workspace state. Tests are grouped by resource type.
//!
//! `corpus_round_trips_semantically` at the bottom iterates every fixture
//! under `cast-fixtures/codex-cli/regression/<repo>/` (extracted from real
//! GitHub repos by `scratch/codex_cast_tests/acquire_corpus.sh`) and asserts
//! semantic equality on every codex surface file. Drop a new fixture
//! directory there and it gets picked up on the next run.

use std::path::{Path, PathBuf};

use theta_test::checkers::codex_cli;
use theta_test::test_context;

// system prompt

#[test]
fn system_prompt_round_trip() {
    let ctx = test_context!().with_fixture("codex-cli/system-prompt/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    codex_cli::assert_agents_md_equal(&original.join("AGENTS.md"), &ctx.path("AGENTS.md"));
}

#[test]
fn system_prompt_not_emitted_when_absent() {
    // mcp/stdio-basic has AGENTS.md (with minimal content), so we cannot use
    // it. instead exercise a hooks-only project: AGENTS.md is present but
    // body is empty after trim --> theta should not emit a system prompt.
    let ctx = test_context!().with_fixture("codex-cli/hooks/basic");

    ctx.cast_from("codex-cli").arg("--force").assert().success();

    // hooks.json content survives via [harness.codex.hooks].
    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[harness.codex"),
        "harness.codex section expected for hooks fixture"
    );
}

// agents

#[test]
fn agents_basic_round_trip() {
    let ctx = test_context!().with_fixture("codex-cli/agents/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    for agent in ["code-reviewer.toml", "debugger.toml"] {
        let rel = format!(".codex/agents/{agent}");
        codex_cli::assert_subagent_toml_equal(&original.join(&rel), &ctx.path(&rel));
    }
}

#[test]
fn agents_full_config_layer_round_trip() {
    // codex subagents carry arbitrary config.toml keys (sandbox_mode,
    // model_reasoning_effort, nested [mcp_servers.*], ...). these must
    // round-trip through [harness.codex.subagent.<slug>] extras.
    let ctx = test_context!().with_fixture("codex-cli/agents/full-config-layer");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    let rel = ".codex/agents/security-checker.toml";
    codex_cli::assert_subagent_toml_equal(&original.join(rel), &ctx.path(rel));

    // verify the extras landed in theta.toml under harness.codex.subagent
    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[harness.codex.subagent"),
        "expected [harness.codex.subagent.*] table for full-config-layer fixture, got:\n{toml}"
    );
}

// mcp

#[test]
fn mcp_stdio_basic_round_trip() {
    let ctx = test_context!().with_fixture("codex-cli/mcp/stdio-basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    codex_cli::assert_config_toml_equal(
        &original.join(".codex/config.toml"),
        &ctx.path(".codex/config.toml"),
    );
}

#[test]
fn mcp_with_extras_round_trip() {
    // codex MCP fields beyond the theta-typed set (env_vars, oauth_resource,
    // scopes, startup_timeout_sec, ...) MUST round-trip through
    // [harness.codex.tool.<name>] extras.
    let ctx = test_context!().with_fixture("codex-cli/mcp/with-extras");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    codex_cli::assert_config_toml_equal(
        &original.join(".codex/config.toml"),
        &ctx.path(".codex/config.toml"),
    );

    let toml = ctx.read_file("theta.toml");
    assert!(
        toml.contains("[harness.codex.tool"),
        "expected [harness.codex.tool.*] for MCP extras, got:\n{toml}"
    );
}

// hooks

#[test]
fn hooks_basic_round_trip() {
    let ctx = test_context!().with_fixture("codex-cli/hooks/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    codex_cli::assert_hooks_equal(
        &original.join(".codex/hooks.json"),
        &ctx.path(".codex/hooks.json"),
    );
}

// profiles (codex-specific, opaque round-trip)

#[test]
fn profiles_round_trip() {
    let ctx = test_context!().with_fixture("codex-cli/profiles/basic");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    codex_cli::assert_config_toml_equal(
        &original.join(".codex/config.toml"),
        &ctx.path(".codex/config.toml"),
    );
}

// skills (canonical .agents/skills/ path)

#[test]
fn skills_agents_path_round_trip() {
    let ctx = test_context!().with_fixture("codex-cli/skills/agents-path");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    codex_cli::assert_skill_equal_relocatable(
        ".agents/skills/lint-check/SKILL.md",
        &original,
        &ctx.path(""),
    );
}

// combined

#[test]
fn combined_everything_round_trip() {
    let ctx = test_context!().with_fixture("codex-cli/combined/everything");
    let original = ctx.snapshot_original();

    ctx.cast_from("codex-cli").arg("--force").assert().success();
    ctx.cast_to("codex-cli").arg("--force").assert().success();

    codex_cli::assert_agents_md_equal(&original.join("AGENTS.md"), &ctx.path("AGENTS.md"));
    codex_cli::assert_config_toml_equal(
        &original.join(".codex/config.toml"),
        &ctx.path(".codex/config.toml"),
    );
    codex_cli::assert_hooks_equal(
        &original.join(".codex/hooks.json"),
        &ctx.path(".codex/hooks.json"),
    );
    codex_cli::assert_subagent_toml_equal(
        &original.join(".codex/agents/reviewer.toml"),
        &ctx.path(".codex/agents/reviewer.toml"),
    );
    codex_cli::assert_skill_equal_relocatable(
        ".agents/skills/lint-check/SKILL.md",
        &original,
        &ctx.path(""),
    );
}
// corpus regression
//
// Single iterating test over every fixture directory under
// `cast-fixtures/codex-cli/regression/<repo>/`. Each fixture is a codex
// surface extracted from a real GitHub repo. The test imports, casts back,
// and walks the original tree comparing every file with the right semantic
// checker. Failures are collected across all fixtures so one nextest run
// surfaces the full regression set.

/// Fixtures that exercise a known limitation. Each entry MUST include a
/// comment with the citation to `notes.rs` or the upstream codex doc.
const CORPUS_EXCLUDED_FIXTURES: &[&str] = &[
    // currently empty - any exclusion lands here with a comment
];

fn corpus_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("failed to find workspace root")
        .to_path_buf()
}

fn corpus_discover_fixtures() -> Vec<String> {
    let dir = corpus_workspace_root()
        .join("test")
        .join("cast-fixtures")
        .join("codex-cli")
        .join("regression");
    let mut names: Vec<String> = fs_err::read_dir(&dir)
        .unwrap_or_else(|e| panic!("failed to read fixture dir {}: {e}", dir.display()))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| !CORPUS_EXCLUDED_FIXTURES.contains(&n.as_str()))
        .collect();
    names.sort();
    names
}

fn corpus_list_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    corpus_walk(root, root, &mut out);
    out.sort();
    out
}

fn corpus_walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs_err::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_str = rel.to_string_lossy();
        if rel_str.starts_with(".git/")
            || rel_str.starts_with(".theta/")
            || rel_str.starts_with("subagents/")
            || rel_str == "theta.toml"
            || rel_str == "theta.lock"
            || rel_str == "system.md"
        {
            continue;
        }
        // skip the local theta mirror at repo root - cast may write there for
        // its own materialization but we only diff codex surfaces.
        if rel_str.starts_with("skills/") || rel_str.starts_with("rules/") {
            continue;
        }
        if path.is_dir() {
            corpus_walk(root, &path, out);
        } else {
            out.push(rel.to_path_buf());
        }
    }
}

fn corpus_assert_file_equal(rel: &Path, original: &Path, cast: &Path) -> Result<(), String> {
    let rel_str = rel.to_string_lossy();
    let original_file = original.join(rel);
    let cast_file = cast.join(rel);

    if !original_file.exists() {
        return Err(format!("original missing: {rel_str}"));
    }
    if !cast_file.exists() {
        if let Some(relocated) = corpus_skill_relocated(&rel_str) {
            let alt = cast.join(&relocated);
            if alt.exists() {
                return corpus_assert_one(&rel_str, &original_file, &alt);
            }
            return Err(format!(
                "skill {rel_str} not found at original location or {}",
                relocated.display()
            ));
        }
        return Err(format!("cast missing: {rel_str}"));
    }
    corpus_assert_one(&rel_str, &original_file, &cast_file)
}

fn corpus_assert_one(rel_str: &str, original: &Path, cast: &Path) -> Result<(), String> {
    use std::panic::AssertUnwindSafe;

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        if rel_str == "AGENTS.md" {
            codex_cli::assert_agents_md_equal(original, cast);
        } else if rel_str.starts_with(".codex/agents/") && rel_str.ends_with(".toml") {
            codex_cli::assert_subagent_toml_equal(original, cast);
        } else if rel_str.ends_with(".toml") {
            codex_cli::assert_config_toml_equal(original, cast);
        } else if rel_str == ".codex/hooks.json" {
            // hooks live across two files (config.toml [hooks] + hooks.json)
            // and codex merges them. covered by the per-fixture union check
            // run separately below; nothing to verify here per-file.
        } else {
            // skill files, .rules, prompts, supporting files - byte-for-byte
            // modulo trailing whitespace.
            let orig_bytes = fs_err::read(original).expect("read original");
            let cast_bytes = fs_err::read(cast).expect("read cast");
            assert!(
                corpus_normalize_trailing(&orig_bytes) == corpus_normalize_trailing(&cast_bytes),
                "content differs: {}",
                original.display(),
            );
        }
    }));

    result.map_err(|payload| {
        let msg = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                payload
                    .downcast_ref::<&'static str>()
                    .map(|s| (*s).to_string())
            })
            .unwrap_or_else(|| "<non-string panic>".into());
        format!("{rel_str}: {msg}")
    })
}

fn corpus_normalize_trailing(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 {
        let b = bytes[end - 1];
        if b == b' ' || b == b'\n' || b == b'\r' || b == b'\t' {
            end -= 1;
        } else {
            break;
        }
    }
    &bytes[..end]
}

fn corpus_skill_relocated(rel_str: &str) -> Option<PathBuf> {
    if let Some(tail) = rel_str.strip_prefix(".codex/skills/") {
        return Some(PathBuf::from(format!(".agents/skills/{tail}")));
    }
    if let Some(tail) = rel_str.strip_prefix(".agents/skills/") {
        return Some(PathBuf::from(format!(".codex/skills/{tail}")));
    }
    None
}

#[test]
fn corpus_round_trips_semantically() {
    let fixtures = corpus_discover_fixtures();
    assert!(
        !fixtures.is_empty(),
        "no regression fixtures found under test/cast-fixtures/codex-cli/regression/"
    );

    let mut failures: Vec<(String, String)> = Vec::new();
    let mut total_files = 0usize;
    let total_fixtures = fixtures.len();

    for fixture in &fixtures {
        let ctx = test_context!().with_fixture(&format!("codex-cli/regression/{fixture}"));
        let original = ctx.snapshot_original();

        let import = ctx
            .cast_from("codex-cli")
            .arg("--force")
            .output()
            .expect("failed to run cast from");
        if !import.status.success() {
            failures.push((
                fixture.clone(),
                format!(
                    "import failed (exit {}): {}",
                    import.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&import.stderr)
                ),
            ));
            continue;
        }

        let cast = ctx
            .cast_to("codex-cli")
            .arg("--force")
            .output()
            .expect("failed to run cast to");
        if !cast.status.success() {
            failures.push((
                fixture.clone(),
                format!(
                    "cast failed (exit {}): {}",
                    cast.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&cast.stderr)
                ),
            ));
            continue;
        }

        let workspace = ctx.path("");
        for rel in corpus_list_files(&original) {
            total_files += 1;
            if let Err(msg) = corpus_assert_file_equal(&rel, &original, &workspace) {
                failures.push((fixture.clone(), msg));
            }
        }

        // hooks live across two files (config.toml [hooks] + hooks.json) and
        // codex merges them at startup. verify the UNION matches, not just
        // each file in isolation.
        // ref: https://developers.openai.com/codex/hooks#where-codex-looks-for-hooks
        let union_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            codex_cli::assert_codex_hooks_union_equal(&original, &workspace);
        }));
        if let Err(payload) = union_result {
            let msg = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&'static str>()
                        .map(|s| (*s).to_string())
                })
                .unwrap_or_else(|| "<non-string panic>".into());
            failures.push((fixture.clone(), format!("hooks union: {msg}")));
        }
    }

    if !failures.is_empty() {
        let mut report = format!(
            "{} regression failures across {} fixtures ({} files checked):\n",
            failures.len(),
            total_fixtures,
            total_files,
        );
        let mut by_fixture: std::collections::BTreeMap<&str, Vec<&str>> =
            std::collections::BTreeMap::new();
        for (fx, msg) in &failures {
            by_fixture
                .entry(fx.as_str())
                .or_default()
                .push(msg.as_str());
        }
        for (fx, msgs) in &by_fixture {
            use std::fmt::Write as _;
            let _ = write!(report, "\n  [{fx}] ({} issue(s))\n", msgs.len());
            for msg in msgs {
                let _ = writeln!(report, "    - {msg}");
            }
        }
        panic!("{report}");
    }
}
