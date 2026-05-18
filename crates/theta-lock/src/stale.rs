use std::fmt;
use std::path::{Path, PathBuf};

use crate::error::ManifestHashError;
use crate::hash::{content_hash, manifest_hash, skill_content_hash};
use crate::types::{ContentHash, LockFile, LockedSource, ResourceLock};

/// Why the lock file is stale — reports the first detected reason;
/// there may be additional stale resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleReason {
    /// `theta.toml` itself was modified
    ManifestChanged,
    /// The system instruction file changed on disk
    SystemInstructionChanged,
    /// A rule file changed on disk
    RuleChanged {
        /// Which rule
        name: String,
    },
    /// A skill directory changed on disk
    SkillChanged {
        /// Which skill
        name: String,
    },
    /// A subagent source changed on disk
    SubagentChanged {
        /// Which subagent
        name: String,
    },
}

impl fmt::Display for StaleReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestChanged => write!(f, "theta.toml changed"),
            Self::SystemInstructionChanged => write!(f, "system instruction changed on disk"),
            Self::RuleChanged { name } => write!(f, "rule '{name}' changed on disk"),
            Self::SkillChanged { name } => write!(f, "skill '{name}' changed on disk"),
            Self::SubagentChanged { name } => write!(f, "subagent '{name}' changed on disk"),
        }
    }
}

/// Check whether the lock is stale.
///
/// Returns `Some(reason)` with the first staleness indicator found,
/// or `None` if fresh.
pub fn is_stale(
    lock: &LockFile,
    current_manifest_bytes: &[u8],
    project_dir: &Path,
) -> Result<Option<StaleReason>, ManifestHashError> {
    let current = manifest_hash(current_manifest_bytes)?;
    if lock.meta.manifest_hash != current {
        return Ok(Some(StaleReason::ManifestChanged));
    }
    Ok(sources_changed(lock, project_dir))
}

fn sources_changed(lock: &LockFile, project_dir: &Path) -> Option<StaleReason> {
    if let Some(ref instr) = lock.instructions {
        if instr
            .system
            .as_ref()
            .is_some_and(|s| source_hash_changed(s, project_dir))
        {
            return Some(StaleReason::SystemInstructionChanged);
        }
        for (name, entry) in &instr.rules {
            if source_hash_changed(entry, project_dir) {
                return Some(StaleReason::RuleChanged { name: name.clone() });
            }
        }
    }
    for (name, entry) in &lock.skills {
        if skill_source_changed(entry, project_dir) {
            return Some(StaleReason::SkillChanged { name: name.clone() });
        }
    }
    for (name, entry) in &lock.subagents {
        if source_hash_changed(entry.as_resource_lock(), project_dir) {
            return Some(StaleReason::SubagentChanged { name: name.clone() });
        }
    }
    None
}

fn source_hash_changed(entry: &ResourceLock, project_dir: &Path) -> bool {
    resolve_and_check(entry, project_dir, store_rule_path, |p| {
        fs_err::read(p).map(|data| content_hash(&data)).ok()
    })
}

fn skill_source_changed(entry: &ResourceLock, project_dir: &Path) -> bool {
    resolve_and_check(entry, project_dir, store_skill_path, |p| {
        skill_content_hash(p).ok()
    })
}

/// Common staleness check: resolve a locked source to a path, hash it,
/// compare against the locked hash. Returns `true` if stale.
fn resolve_and_check(
    entry: &ResourceLock,
    project_dir: &Path,
    system_path: impl FnOnce(&str) -> Option<PathBuf>,
    hash_at: impl FnOnce(&Path) -> Option<ContentHash>,
) -> bool {
    let resolved = match &entry.source {
        LockedSource::Path { path } => Some(project_dir.join(path)),
        LockedSource::System { system } => system_path(system),
        _ => return false, // remote sources: can't check locally
    };
    let Some(path) = resolved else {
        return true;
    };
    match hash_at(&path) {
        Some(hash) => hash != entry.content_hash,
        None => true, // missing = stale
    }
}

fn store_rule_path(name: &str) -> Option<PathBuf> {
    let data_dir = theta_dirs::data_dir()?;
    Some(theta_static::SystemStoreLayout::new(&data_dir).rule(name))
}

fn store_skill_path(name: &str) -> Option<PathBuf> {
    let data_dir = theta_dirs::data_dir()?;
    Some(theta_static::SystemStoreLayout::new(&data_dir).skill(name))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::hash::{content_hash, manifest_hash};
    use crate::stale::is_stale;
    use crate::types::*;

    #[test]
    fn detects_manifest_change() {
        let manifest_v1 = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"v1\"\nversion = \"0.1.0\"\n";
        let manifest_v2 = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"v2 changed\"\nversion = \"0.1.0\"\n";

        let tmp = std::env::temp_dir();
        let lock = LockFile {
            meta: LockMeta {
                schema: "2026-04".into(),
                manifest_hash: manifest_hash(manifest_v1).unwrap(),
            },
            instructions: None,
            skills: BTreeMap::new(),
            subagents: BTreeMap::new(),
        };
        assert_eq!(
            is_stale(&lock, manifest_v2, &tmp).unwrap(),
            Some(super::StaleReason::ManifestChanged),
        );
        assert_eq!(is_stale(&lock, manifest_v1, &tmp).unwrap(), None);
    }

    #[test]
    fn detects_source_content_change() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path();

        let rules_dir = project.join("instructions/rules");
        fs_err::create_dir_all(&rules_dir).unwrap();
        let rule_path = rules_dir.join("check.md");
        fs_err::write(&rule_path, "original content").unwrap();

        let manifest_bytes = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"d\"\nversion = \"0.1.0\"\n";

        let lock = LockFile {
            meta: LockMeta {
                schema: "2026-04".into(),
                manifest_hash: manifest_hash(manifest_bytes).unwrap(),
            },
            instructions: Some(InstructionsLock {
                system: None,
                rules: BTreeMap::from([(
                    "check".into(),
                    ResourceLock {
                        source: LockedSource::Path {
                            path: "instructions/rules/check.md".into(),
                        },
                        content_hash: content_hash(b"original content"),
                    },
                )]),
            }),
            skills: BTreeMap::new(),
            subagents: BTreeMap::new(),
        };

        assert_eq!(is_stale(&lock, manifest_bytes, project).unwrap(), None);

        fs_err::write(&rule_path, "edited content").unwrap();
        assert_eq!(
            is_stale(&lock, manifest_bytes, project).unwrap(),
            Some(super::StaleReason::RuleChanged {
                name: "check".into()
            }),
        );

        fs_err::remove_file(&rule_path).unwrap();
        assert_eq!(
            is_stale(&lock, manifest_bytes, project).unwrap(),
            Some(super::StaleReason::RuleChanged {
                name: "check".into()
            }),
        );
    }

    #[test]
    fn detects_skill_content_change() {
        use crate::hash::skill_content_hash;

        let dir = tempfile::tempdir().unwrap();
        let project = dir.path();

        let skill_dir = project.join("skills/my-skill");
        fs_err::create_dir_all(&skill_dir).unwrap();
        fs_err::write(skill_dir.join("SKILL.md"), "original skill content").unwrap();

        let manifest_bytes = b"[theta]\nschema = \"2026-04\"\n\n[agent]\nname = \"test\"\ndescription = \"d\"\nversion = \"0.1.0\"\n\n[skills.my-skill]\nsource = { path = \"skills/my-skill\" }\n";

        let lock = LockFile {
            meta: LockMeta {
                schema: "2026-04".into(),
                manifest_hash: manifest_hash(manifest_bytes).unwrap(),
            },
            instructions: None,
            skills: BTreeMap::from([(
                "my-skill".into(),
                ResourceLock {
                    source: LockedSource::Path {
                        path: "skills/my-skill".into(),
                    },
                    content_hash: skill_content_hash(&skill_dir).unwrap(),
                },
            )]),
            subagents: BTreeMap::new(),
        };

        assert_eq!(is_stale(&lock, manifest_bytes, project).unwrap(), None);

        fs_err::write(skill_dir.join("SKILL.md"), "edited skill content").unwrap();
        assert_eq!(
            is_stale(&lock, manifest_bytes, project).unwrap(),
            Some(super::StaleReason::SkillChanged {
                name: "my-skill".into()
            }),
        );

        fs_err::remove_dir_all(&skill_dir).unwrap();
        assert_eq!(
            is_stale(&lock, manifest_bytes, project).unwrap(),
            Some(super::StaleReason::SkillChanged {
                name: "my-skill".into()
            }),
        );
    }
}
