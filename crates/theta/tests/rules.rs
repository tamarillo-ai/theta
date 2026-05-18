//! Rule validation tests.

use std::collections::BTreeMap;

use theta_schema::{ApplyMode, Instructions, LocalOrGitRef, LocalPathRef, Rule, Validate};

#[test]
fn rule_name_validation() {
    // simple kebab names
    assert!(theta_schema::is_valid_rule_name("safety"));
    assert!(theta_schema::is_valid_rule_name("code-style"));
    assert!(theta_schema::is_valid_rule_name("rust-2024"));
    assert!(!theta_schema::is_valid_rule_name("-leading"));
    assert!(!theta_schema::is_valid_rule_name("trailing-"));
    assert!(!theta_schema::is_valid_rule_name("UPPER"));
    assert!(!theta_schema::is_valid_rule_name("has space"));
    assert!(!theta_schema::is_valid_rule_name(""));

    // path-qualified names (group/rule)
    assert!(theta_schema::is_valid_rule_name("review/pr-review"));
    assert!(theta_schema::is_valid_rule_name("backend/typescript"));
    assert!(theta_schema::is_valid_rule_name("a/b"));
    assert!(theta_schema::is_valid_rule_name("a/b/c")); // valid but warns at ≥3 depth
    assert!(!theta_schema::is_valid_rule_name("/leading"));
    assert!(!theta_schema::is_valid_rule_name("trailing/"));
    assert!(!theta_schema::is_valid_rule_name("a//b"));
    assert!(!theta_schema::is_valid_rule_name("a/"));
    assert!(!theta_schema::is_valid_rule_name("a/UPPER"));
    assert!(!theta_schema::is_valid_rule_name("a/-bad"));
}

#[test]
fn rule_validate_model_decision_requires_description() {
    let rule = Rule {
        src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/test.md")),
        summary: None,
        description: None,
        apply: ApplyMode::ModelDecision,
        apply_to: None,
    };
    let mut diags = Vec::new();
    rule.validate_named("no-desc", &mut diags);
    assert!(diags.iter().any(|d| d.message.contains("description")));
}

#[test]
fn rule_validate_glob_without_apply_to_warns() {
    let rule = Rule {
        src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/test.md")),
        summary: None,
        description: Some("test".to_string()),
        apply: ApplyMode::Glob,
        apply_to: None,
    };
    let mut diags = Vec::new();
    rule.validate_named("no-patterns", &mut diags);
    assert!(diags.iter().any(|d| d.message.contains("apply_to")));
}

#[test]
fn rule_validate_always_clean() {
    let rule = Rule {
        src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/test.md")),
        summary: None,
        description: Some("Safety guardrails".to_string()),
        apply: ApplyMode::Always,
        apply_to: None,
    };
    let mut diags = Vec::new();
    rule.validate_named("safety", &mut diags);
    assert!(diags.is_empty());
}

#[test]
fn instructions_validate_rejects_bad_rule_name() {
    let mut rules = BTreeMap::new();
    rules.insert(
        "INVALID_NAME".to_string(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/test.md")),
            summary: None,
            description: None,
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    let instructions = Instructions {
        system: None,
        rules: Some(rules),
    };
    let mut diags = Vec::new();
    instructions.validate(&mut diags);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("INVALID_NAME") && d.message.contains("not a valid"))
    );
}
#[test]
fn path_qualified_rule_name_accepted_by_instructions_validate() {
    let mut rules = BTreeMap::new();
    rules.insert(
        "review/pr-review".to_string(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/review/pr-review.md")),
            summary: None,
            description: None,
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    let instructions = Instructions {
        system: None,
        rules: Some(rules),
    };
    let mut diags = Vec::new();
    instructions.validate(&mut diags);
    // no errors - path-qualified names are valid
    assert!(
        !diags
            .iter()
            .any(|d| d.level == theta_schema::DiagLevel::Error),
        "unexpected error: {diags:?}"
    );
}

#[test]
fn deeply_nested_rule_name_warns() {
    let mut rules = BTreeMap::new();
    rules.insert(
        "a/b/c".to_string(),
        Rule {
            src: LocalOrGitRef::Local(LocalPathRef::from("rules/a/b/c.md")),
            summary: None,
            description: None,
            apply: ApplyMode::Always,
            apply_to: None,
        },
    );
    let instructions = Instructions {
        system: None,
        rules: Some(rules),
    };
    let mut diags = Vec::new();
    instructions.validate(&mut diags);
    assert!(
        diags.iter().any(|d| d.message.contains("nesting levels")),
        "expected depth warning, got: {diags:?}"
    );
}

#[test]
fn flatten_rule_name_works() {
    use theta_harness::layout::flatten_rule_name;
    assert_eq!(flatten_rule_name("safety"), "safety");
    assert_eq!(flatten_rule_name("review/pr-review"), "review-pr-review");
    assert_eq!(
        flatten_rule_name("backend/typescript"),
        "backend-typescript"
    );
    assert_eq!(flatten_rule_name("a/b/c"), "a-b-c");
}
