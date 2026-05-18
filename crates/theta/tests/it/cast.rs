use assert_fs::prelude::*;
use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn cast_to_unknown_harness() {
    let ctx = test_context!();
    ctx.cast_to("doesnotexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn cast_from_unknown_harness() {
    let ctx = test_context!();
    ctx.cast_from("doesnotexist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn manifest_discovered_from_subdirectory() {
    let ctx = test_context!();
    ctx.temp_dir
        .child("theta.toml")
        .write_str(
            r#"
[theta]
schema = "2026-04"

[agent]
name = "subdir-test"
description = "testing manifest discovery"
"#,
        )
        .unwrap();

    let subdir = ctx.temp_dir.child("src/utils");
    subdir.create_dir_all().unwrap();

    let mut cmd = ctx.command();
    cmd.current_dir(subdir.path());
    cmd.args(["check"]);
    cmd.assert().success();
}

#[test]
fn cast_works_from_subdirectory() {
    let ctx = test_context!();
    ctx.temp_dir
        .child("theta.toml")
        .write_str(
            r#"
[theta]
schema = "2026-04"

[agent]
name = "subdir-cast"
description = "testing cast from subdirectory"

[instructions]
system = "instructions/system.md"
"#,
        )
        .unwrap();
    ctx.temp_dir
        .child("instructions/system.md")
        .write_str("you are a helpful assistant")
        .unwrap();

    let subdir = ctx.temp_dir.child("some/nested/dir");
    subdir.create_dir_all().unwrap();

    let mut cmd = ctx.command();
    cmd.current_dir(subdir.path());
    cmd.args(["cast", "to", "copilot"]);
    cmd.assert().success();
}
#[test]
fn cast_to_with_output_dir() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.command()
        .args(["add", "system", "--content", "You help."])
        .assert()
        .success();

    let out = ctx.temp_dir.child("cast-output");
    ctx.cast_to("copilot")
        .args(["--output", out.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(out.path().exists());
}

#[test]
fn cast_to_notes() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();
    ctx.command()
        .args(["add", "system", "--content", "You help."])
        .assert()
        .success();

    ctx.cast_to("copilot").arg("--notes").assert().success();
}

#[test]
fn cast_from_with_input_dir() {
    let ctx = test_context!();

    // create a minimal claude-code project to import from
    let src = ctx.temp_dir.child("source-project");
    src.create_dir_all().unwrap();
    src.child("CLAUDE.md")
        .write_str("# Agent\nYou are helpful.")
        .unwrap();

    ctx.cast_from("claude-code")
        .args(["--input", src.path().to_str().unwrap()])
        .assert()
        .success();

    // theta.toml should exist - either in workspace or in input dir
    let in_workspace = ctx.temp_dir.child("theta.toml").path().exists();
    let in_source = src.child("theta.toml").path().exists();
    assert!(
        in_workspace || in_source,
        "theta.toml should be created somewhere"
    );
}
