use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn sync_materializes_files() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("Be helpful.")
        .assert()
        .success();

    ctx.command()
        .args(["add", "rule", "safe"])
        .arg("--content")
        .arg("Be safe.")
        .assert()
        .success();

    ctx.sync().assert().success();

    let theta_dir = ctx.temp_dir.child(".theta");
    theta_dir.assert(predicates::path::exists());

    let system = ctx.temp_dir.child(".theta/system.md");
    system.assert(predicates::path::exists());
    let content = fs_err::read_to_string(system.path()).unwrap();
    assert!(content.contains("Be helpful"));

    let rule = ctx.temp_dir.child(".theta/rules/safe.md");
    rule.assert(predicates::path::exists());
    let content = fs_err::read_to_string(rule.path()).unwrap();
    assert!(content.contains("Be safe"));
}

#[test]
fn sync_is_idempotent() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("Be helpful.")
        .assert()
        .success();

    ctx.sync().assert().success();

    ctx.sync()
        .assert()
        .success()
        .stderr(predicate::str::contains("up to date"));
}

#[test]
fn sync_force_rematerializes() {
    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("test-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system"])
        .arg("--content")
        .arg("Be helpful.")
        .assert()
        .success();

    ctx.sync().assert().success();

    ctx.sync().arg("--force").assert().success();
}

#[test]
fn sync_materializes_path_qualified_rule() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.command()
        .args(["add", "rule", "review/pr-review", "--content", "Check PRs."])
        .assert()
        .success();

    ctx.sync().assert().success();

    let rule = ctx.temp_dir.child(".theta/rules/review/pr-review.md");
    rule.assert(predicates::path::exists());
    let content = fs_err::read_to_string(rule.path()).unwrap();
    assert!(content.contains("Check PRs"));
}

#[test]
fn sync_fails_when_materialized_skill_content_is_invalid() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("skill")
        .args(["analysis", "--no-sync"])
        .assert()
        .success();

    let skill = ctx.temp_dir.child("skills/analysis/SKILL.md");
    fs_err::write(skill.path(), "---\nname: analysis\n---\n# analysis\n").unwrap();

    ctx.sync()
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "materialized but content validation failed",
        ))
        .stderr(predicate::str::contains(
            "SKILL.md frontmatter is missing required `description` field",
        ));
}

#[test]
fn sync_theta_out_dir_redirects_theta_dir_and_lock() {
    use assert_fs::TempDir;
    use assert_fs::fixture::PathChild;

    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("out-dir-agent")
        .assert()
        .success();

    ctx.command()
        .args(["add", "system", "--content", "Out-dir system prompt."])
        .assert()
        .success();

    ctx.command()
        .args(["add", "rule", "safety", "--content", "Be safe."])
        .assert()
        .success();

    // Create a separate output directory.
    let out_dir = TempDir::new().expect("failed to create out dir");

    // Sync with THETA_OUT_DIR pointing elsewhere.
    ctx.sync()
        .env("THETA_OUT_DIR", out_dir.path())
        .assert()
        .success();

    // .theta/ and theta.lock land in THETA_OUT_DIR, not in the project.
    let out_theta = out_dir.child(".theta");
    out_theta.assert(predicates::path::exists());

    let out_system = out_dir.child(".theta/system.md");
    out_system.assert(predicates::path::exists());
    let content = fs_err::read_to_string(out_system.path()).unwrap();
    assert!(content.contains("Out-dir system prompt"));

    let out_rule = out_dir.child(".theta/rules/safety.md");
    out_rule.assert(predicates::path::exists());
    let content = fs_err::read_to_string(out_rule.path()).unwrap();
    assert!(content.contains("Be safe"));

    let out_lock = out_dir.child("theta.lock");
    out_lock.assert(predicates::path::exists());

    // Original project must NOT have a .theta/ or theta.lock from this sync.
    ctx.temp_dir
        .child(".theta")
        .assert(predicates::path::missing());
    ctx.temp_dir
        .child("theta.lock")
        .assert(predicates::path::missing());
}

#[test]
fn sync_theta_out_dir_with_external_manifest() {
    use assert_fs::TempDir;
    use assert_fs::fixture::PathChild;

    let ctx = test_context!();
    ctx.init()
        .arg("--name")
        .arg("ext-manifest-agent")
        .assert()
        .success();

    ctx.command()
        .args([
            "add",
            "system",
            "--content",
            "External manifest system prompt.",
        ])
        .assert()
        .success();

    let out_dir = TempDir::new().expect("failed to create out dir");
    let manifest_path = ctx.temp_dir.child("theta.toml");

    // Sync using --manifest pointing at the real project but THETA_OUT_DIR elsewhere.
    ctx.command()
        .args(["sync", "--manifest"])
        .arg(manifest_path.path())
        .env("THETA_OUT_DIR", out_dir.path())
        .current_dir(out_dir.path())
        .assert()
        .success();

    // Materialized output lands in THETA_OUT_DIR.
    out_dir
        .child(".theta/system.md")
        .assert(predicates::path::exists());

    let content = fs_err::read_to_string(out_dir.child(".theta/system.md").path()).unwrap();
    assert!(content.contains("External manifest system prompt"));

    // Lock lands in THETA_OUT_DIR.
    out_dir
        .child("theta.lock")
        .assert(predicates::path::exists());
}
