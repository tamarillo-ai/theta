use assert_fs::assert::PathAssert;
use assert_fs::fixture::PathChild;
use theta_test::test_context;

#[test]
fn creates_file_and_updates_manifest() {
    let ctx = test_context!();
    ctx.init().arg("--name").arg("t").assert().success();

    ctx.add("system").assert().success();

    let manifest = fs_err::read_to_string(ctx.temp_dir.child("theta.toml").path()).unwrap();
    assert!(manifest.contains("system"));

    ctx.temp_dir
        .child("instructions/system.md")
        .assert(predicates::path::exists());
}
