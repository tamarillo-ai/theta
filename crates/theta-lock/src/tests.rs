//! Integration tests: serialization round-trips.

use std::collections::BTreeMap;

use crate::*;

#[test]
fn lock_round_trips_through_toml() {
    let lock = LockFile {
        meta: LockMeta {
            schema: "2026-04".into(),
            manifest_hash:
                "sha256:0000000000000000000000000000000000000000000000000000000000abc123"
                    .parse()
                    .unwrap(),
        },
        instructions: Some(InstructionsLock {
            system: Some(ResourceLock {
                source: LockedSource::Path {
                    path: "instructions/system.md".into(),
                },
                content_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000111"
                        .parse()
                        .unwrap(),
            }),
            rules: BTreeMap::from([(
                "safety".into(),
                ResourceLock {
                    source: LockedSource::Path {
                        path: "instructions/rules/safety.md".into(),
                    },
                    content_hash:
                        "sha256:0000000000000000000000000000000000000000000000000000000000000222"
                            .parse()
                            .unwrap(),
                },
            )]),
        }),
        skills: BTreeMap::from([(
            "osint".into(),
            ResourceLock {
                source: LockedSource::Path {
                    path: "./skills/osint".into(),
                },
                content_hash:
                    "sha256:0000000000000000000000000000000000000000000000000000000000000333"
                        .parse()
                        .unwrap(),
            },
        )]),
        subagents: BTreeMap::from([(
            "researcher".into(),
            SubagentLock::Ref {
                resource: ResourceLock {
                    source: LockedSource::Path {
                        path: "./agents/researcher.theta.toml".into(),
                    },
                    content_hash:
                        "sha256:0000000000000000000000000000000000000000000000000000000000000444"
                            .parse()
                            .unwrap(),
                },
                instructions: None,
                skills: BTreeMap::new(),
            },
        )]),
    };

    let serialized = toml::to_string_pretty(&lock).unwrap();
    let parsed: LockFile = toml::from_str(&serialized).unwrap();
    assert_eq!(lock, parsed);
}

#[test]
fn lock_round_trips_with_inline_subagent() {
    let lock = LockFile {
        meta: LockMeta {
            schema: "2026-04".into(),
            manifest_hash: "sha256:0000000000000000000000000000000000000000000000000000000000000abc".parse().unwrap(),
        },
        instructions: None,
        skills: BTreeMap::new(),
        subagents: BTreeMap::from([
            (
                "ref-agent".into(),
                SubagentLock::Ref {
                    resource: ResourceLock {
                        source: LockedSource::Path {
                            path: "agents/child.toml".into(),
                        },
                        content_hash: "sha256:0000000000000000000000000000000000000000000000000000000000000444".parse().unwrap(),
                    },
                    instructions: None,
                    skills: BTreeMap::new(),
                },
            ),
            (
                "inline-agent".into(),
                SubagentLock::Inline {
                    prompt: ResourceLock {
                        source: LockedSource::Path {
                            path: "subagents/inline-agent.md".into(),
                        },
                        content_hash: "sha256:0000000000000000000000000000000000000000000000000000000000000bbb".parse().unwrap(),
                    },
                },
            ),
        ]),
    };

    let serialized = toml::to_string_pretty(&lock).unwrap();
    let parsed: LockFile = toml::from_str(&serialized).unwrap();
    assert_eq!(lock, parsed);
}

#[test]
fn lock_round_trips_with_path_qualified_rule() {
    let lock = LockFile {
        meta: LockMeta {
            schema: "2026-04".into(),
            manifest_hash:
                "sha256:0000000000000000000000000000000000000000000000000000000000000fff"
                    .parse()
                    .unwrap(),
        },
        instructions: Some(InstructionsLock {
            system: None,
            rules: BTreeMap::from([
                (
                    "review/pr-review".into(),
                    ResourceLock {
                        source: LockedSource::Path {
                            path: "instructions/rules/review/pr-review.md".into(),
                        },
                        content_hash:
                            "sha256:0000000000000000000000000000000000000000000000000000000000000aaa"
                                .parse()
                                .unwrap(),
                    },
                ),
                (
                    "safety".into(),
                    ResourceLock {
                        source: LockedSource::Path {
                            path: "instructions/rules/safety.md".into(),
                        },
                        content_hash:
                            "sha256:0000000000000000000000000000000000000000000000000000000000000bbb"
                                .parse()
                                .unwrap(),
                    },
                ),
            ]),
        }),
        skills: BTreeMap::new(),
        subagents: BTreeMap::new(),
    };

    let serialized = toml::to_string_pretty(&lock).unwrap();
    assert!(
        serialized.contains("\"review/pr-review\""),
        "path-qualified key must be quoted in TOML output"
    );
    let parsed: LockFile = toml::from_str(&serialized).unwrap();
    assert_eq!(lock, parsed);
    assert!(
        parsed
            .instructions
            .unwrap()
            .rules
            .contains_key("review/pr-review")
    );
}

#[test]
fn old_lockfile_without_prompt_path_deserializes_as_ref() {
    let toml_str = r#"
[meta]
schema = "2026-04"
manifest_hash = "sha256:00000000000000000000000000000000000000000000000000000000000000dd"

[subagents.researcher]
source = { path = "agents/researcher.toml" }
content_hash = "sha256:00000000000000000000000000000000000000000000000000000000000000ee"
"#;
    let lock: LockFile = toml::from_str(toml_str).unwrap();
    assert!(matches!(
        lock.subagents.get("researcher"),
        Some(SubagentLock::Ref { .. })
    ));
}
