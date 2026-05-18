use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use theta_test::test_context;

//  happy path

#[test]
fn creates_file_and_updates_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule").arg("safety").assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("safety"));

    let rule_file = ctx.temp_dir.child("instructions/rules/safety.md");
    rule_file.assert(predicates::path::exists());
}

//  valid argument combinations

#[test]
fn with_summary() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args(["safety", "--summary", "Code safety guidelines"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("Code safety guidelines"));
}

#[test]
fn add_rule_with_description() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args(["docs", "--description", "Documentation rules"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("Documentation rules"));
}

#[test]
fn add_rule_with_apply_always() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args(["perf", "--apply", "always"])
        .assert()
        .success();
}

#[test]
fn add_rule_with_apply_model_decision() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args([
            "optional-lint",
            "--apply",
            "model-decision",
            "--description",
            "Apply when linting",
        ])
        .assert()
        .success();
}

#[test]
fn add_rule_with_apply_glob() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args([
            "ts-only",
            "--apply",
            "glob",
            "--apply-to",
            "*.ts",
            "--apply-to",
            "*.tsx",
        ])
        .assert()
        .success();
}

#[test]
fn add_rule_with_content() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args(["inline", "--content", "Always be concise."])
        .assert()
        .success();

    let rule_file = ctx.temp_dir.child("instructions/rules/inline.md");
    let content = fs_err::read_to_string(rule_file.path()).unwrap();
    assert!(content.contains("Always be concise"));
}

#[test]
fn add_rule_with_path_existing_file() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    let custom = ctx.temp_dir.child("custom-rules/external.md");
    fs_err::create_dir_all(custom.path().parent().unwrap()).unwrap();
    fs_err::write(custom.path(), "# External rule").unwrap();

    ctx.add("rule")
        .args(["external", "--path", "custom-rules/external.md"])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("custom-rules/external.md"));
}

#[test]
fn add_rule_full_combo() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args([
            "full-combo",
            "--summary",
            "All flags",
            "--description",
            "Full flag test",
            "--apply",
            "glob",
            "--apply-to",
            "*.py",
        ])
        .assert()
        .success();
}

//  invalid name patterns

#[test]
fn add_rule_rejects_uppercase() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("UPPER").assert().failure();
}

#[test]
fn add_rule_rejects_spaces() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("bad name").assert().failure();
}

#[test]
fn add_rule_rejects_consecutive_hyphens() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("bad--name").assert().failure();
}

#[test]
fn add_rule_rejects_trailing_hyphen() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("trailing-").assert().failure();
}

#[test]
fn add_rule_rejects_underscores() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("no_underscores").assert().failure();
}

#[test]
fn add_rule_rejects_dots() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("no.dots").assert().failure();
}

#[test]
fn add_rule_rejects_duplicate() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule").arg("safety").assert().success();
    ctx.add("rule").arg("safety").assert().failure();
}

//  argument conflict detection (clap-level, exit code 2)

#[test]
fn add_rule_system_conflicts_with_path() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule")
        .args(["x", "--system", "--path", "foo"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_rule_system_conflicts_with_content() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule")
        .args(["x", "--system", "--content", "bar"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_rule_system_conflicts_with_git() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule")
        .args(["x", "--system", "--git", "https://example.com"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_rule_git_conflicts_with_path() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule")
        .args(["x", "--git", "https://example.com", "--path", "foo"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_rule_branch_and_tag_conflict() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule")
        .args([
            "x",
            "--git",
            "https://example.com",
            "--branch",
            "main",
            "--tag",
            "v1.0",
            "--file",
            "x.md",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_rule_with_branch_writes_correct_toml() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args([
            "remote",
            "--git",
            "https://github.com/org/rules",
            "--branch",
            "main",
            "--file",
            "safety.md",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("branch = \"main\""));
    assert!(!manifest.contains("ref ="));
}

#[test]
fn add_rule_with_tag_writes_correct_toml() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args([
            "pinned",
            "--git",
            "https://github.com/org/rules",
            "--tag",
            "v2.0",
            "--file",
            "safety.md",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("tag = \"v2.0\""));
}

#[test]
fn add_rule_rev_requires_git() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule")
        .args(["x", "--rev", "main"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn add_rule_file_requires_git() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.add("rule")
        .args(["x", "--file", "some.md"])
        .assert()
        .failure()
        .code(2);
}

// path-qualified rule names

#[test]
fn add_path_qualified_rule_creates_subdirectory() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args([
            "review/pr-review",
            "--content",
            "Check for security issues.",
        ])
        .assert()
        .success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(
        manifest.contains("\"review/pr-review\""),
        "path-qualified key should appear in manifest"
    );

    let rule_file = ctx.temp_dir.child("instructions/rules/review/pr-review.md");
    rule_file.assert(predicates::path::exists());
    let content = fs_err::read_to_string(rule_file.path()).unwrap();
    assert!(content.contains("Check for security issues"));
}

#[test]
fn add_deeply_nested_rule_creates_dirs() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("rule")
        .args(["backend/api/validation", "--content", "Validate inputs."])
        .assert()
        .success();

    let rule_file = ctx
        .temp_dir
        .child("instructions/rules/backend/api/validation.md");
    rule_file.assert(predicates::path::exists());
}

#[test]
fn add_path_qualified_rule_rejects_invalid_segment() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    // uppercase segment
    ctx.add("rule").arg("review/UPPER").assert().failure();

    // leading slash
    ctx.add("rule").arg("/leading").assert().failure();

    // trailing slash
    ctx.add("rule").arg("trailing/").assert().failure();

    // empty segment
    ctx.add("rule").arg("a//b").assert().failure();
}
