use predicates::prelude::*;
use theta_test::test_context;

#[test]
fn schema_outputs_json() {
    let ctx = test_context!();
    ctx.command()
        .arg("schema")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"$schema\"").or(predicate::str::contains("\"type\"")));
}
