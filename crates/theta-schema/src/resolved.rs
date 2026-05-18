use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolutionStatus {
    Resolved,
    Deferred,
    Missing,
    Error,
}

#[derive(Debug, Clone)]
struct ResolvedRef {
    status: ResolutionStatus,
    content: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ResolvedRefKey {
    Field {
        manifest_path: &'static str,
    },
    Named {
        table_path: &'static str,
        name: String,
    },
}

impl ResolvedRefKey {
    pub const fn instructions_system() -> Self {
        Self::Field {
            manifest_path: "[instructions].system",
        }
    }

    pub fn instructions_rule(name: impl Into<String>) -> Self {
        Self::Named {
            table_path: "[instructions.rules]",
            name: name.into(),
        }
    }

    pub fn skill(name: impl Into<String>) -> Self {
        Self::Named {
            table_path: "[skills]",
            name: name.into(),
        }
    }

    pub fn subagent_ref(name: impl Into<String>) -> Self {
        Self::Named {
            table_path: "[[subagents]].ref",
            name: name.into(),
        }
    }

    pub fn subagent_prompt(name: impl Into<String>) -> Self {
        Self::Named {
            table_path: "[[subagents]].prompt_path",
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedRefs {
    inner: BTreeMap<ResolvedRefKey, ResolvedRef>,
}

impl ResolvedRefs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: ResolvedRefKey, content: String) {
        self.insert_resolved(key, content);
    }

    pub fn insert_resolved(&mut self, key: ResolvedRefKey, content: String) {
        self.inner.insert(
            key,
            ResolvedRef {
                status: ResolutionStatus::Resolved,
                content: Some(content),
                error: None,
            },
        );
    }

    pub fn insert_deferred(&mut self, key: ResolvedRefKey) {
        self.inner.insert(
            key,
            ResolvedRef {
                status: ResolutionStatus::Deferred,
                content: None,
                error: None,
            },
        );
    }

    pub fn insert_missing(&mut self, key: ResolvedRefKey) {
        self.inner.insert(
            key,
            ResolvedRef {
                status: ResolutionStatus::Missing,
                content: None,
                error: None,
            },
        );
    }

    pub fn insert_error(&mut self, key: ResolvedRefKey, error: impl Into<String>) {
        self.inner.insert(
            key,
            ResolvedRef {
                status: ResolutionStatus::Error,
                content: None,
                error: Some(error.into()),
            },
        );
    }

    pub fn get(&self, key: &ResolvedRefKey) -> Option<&str> {
        self.inner
            .get(key)
            .and_then(|entry| entry.content.as_deref())
    }

    pub fn get_status(&self, key: &ResolvedRefKey) -> Option<ResolutionStatus> {
        self.inner.get(key).map(|entry| entry.status)
    }

    pub fn get_error(&self, key: &ResolvedRefKey) -> Option<&str> {
        self.inner.get(key).and_then(|entry| entry.error.as_deref())
    }

    pub fn insert_instructions_system(&mut self, content: String) {
        self.insert(ResolvedRefKey::instructions_system(), content);
    }

    pub fn insert_instructions_rule(&mut self, name: impl Into<String>, content: String) {
        self.insert(ResolvedRefKey::instructions_rule(name), content);
    }

    pub fn insert_skill(&mut self, name: impl Into<String>, content: String) {
        self.insert(ResolvedRefKey::skill(name), content);
    }

    pub fn get_instructions_system(&self) -> Option<&str> {
        self.get(&ResolvedRefKey::instructions_system())
    }

    pub fn get_instructions_rule(&self, name: &str) -> Option<&str> {
        self.get(&ResolvedRefKey::instructions_rule(name))
    }

    pub fn get_skill(&self, name: &str) -> Option<&str> {
        self.get(&ResolvedRefKey::skill(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_helpers_encode_manifest_identity() {
        let system = ResolvedRefKey::instructions_system();
        let rule = ResolvedRefKey::instructions_rule("safety");
        let skill = ResolvedRefKey::skill("osint");

        assert_eq!(
            system,
            ResolvedRefKey::Field {
                manifest_path: "[instructions].system"
            }
        );
        assert_eq!(
            rule,
            ResolvedRefKey::Named {
                table_path: "[instructions.rules]",
                name: "safety".to_string()
            }
        );
        assert_eq!(
            skill,
            ResolvedRefKey::Named {
                table_path: "[skills]",
                name: "osint".to_string()
            }
        );
    }

    #[test]
    fn refs_are_keyed_by_identity_not_path() {
        let mut refs = ResolvedRefs::new();
        refs.insert_instructions_rule("safety", "Never leak secrets".to_string());

        assert_eq!(
            refs.get_instructions_rule("safety"),
            Some("Never leak secrets")
        );
        assert_eq!(refs.get_instructions_rule("style"), None);
    }

    #[test]
    fn status_tracking_distinguishes_deferred_and_missing() {
        let mut refs = ResolvedRefs::new();
        refs.insert_deferred(ResolvedRefKey::instructions_rule("remote"));
        refs.insert_missing(ResolvedRefKey::instructions_rule("local-missing"));

        assert_eq!(
            refs.get_status(&ResolvedRefKey::instructions_rule("remote")),
            Some(ResolutionStatus::Deferred)
        );
        assert_eq!(
            refs.get_status(&ResolvedRefKey::instructions_rule("local-missing")),
            Some(ResolutionStatus::Missing)
        );
    }
}
