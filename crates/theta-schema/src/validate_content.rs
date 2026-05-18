use crate::{
    Diagnostic, Instructions, ResolutionStatus, ResolvedRefKey, ResolvedRefs, Skill,
    SkillFrontmatter, ThetaManifest, ValidateContent,
};

impl ValidateContent for Instructions {
    fn validate_content(&self, _name: &str, resolved: &ResolvedRefs, diags: &mut Vec<Diagnostic>) {
        if let Some(ref system) = self.system {
            let key = ResolvedRefKey::instructions_system();
            match resolved.get_status(&key) {
                Some(ResolutionStatus::Resolved) => {
                    if let Some(content) = resolved.get_instructions_system() {
                        if content.is_empty() {
                            diags.push(Diagnostic::warn(
                                "[instructions].system",
                                format!("system prompt file is empty: {}", system.as_str()),
                            ));
                        } else if theta_static::is_system_template(content) {
                            diags.push(Diagnostic::warn(
                                "[instructions].system",
                                format!(
                                    "system prompt still using scaffold template - edit {}",
                                    system.as_str()
                                ),
                            ));
                        }
                    }
                }
                Some(ResolutionStatus::Deferred) => {
                    // reachability diagnostics already report deferred remote refs
                }
                Some(ResolutionStatus::Missing) => {
                    diags.push(Diagnostic::error(
                        "[instructions].system",
                        format!(
                            "system prompt content could not be resolved: {}",
                            system.as_str()
                        ),
                    ));
                }
                Some(ResolutionStatus::Error) => {
                    let detail = resolved.get_error(&key).unwrap_or("unknown resolver error");
                    diags.push(Diagnostic::error(
                        "[instructions].system",
                        format!(
                            "system prompt content could not be resolved: {} ({})",
                            system.as_str(),
                            detail
                        ),
                    ));
                }
                None => {
                    diags.push(Diagnostic::error(
                        "[instructions].system",
                        format!(
                            "system prompt content could not be resolved: {}",
                            system.as_str()
                        ),
                    ));
                }
            }
        }
        if let Some(ref rules) = self.rules {
            for (name, rule) in rules {
                let label = format!("[instructions.rules.{name}]");
                let key = ResolvedRefKey::instructions_rule(name.clone());
                match resolved.get_status(&key) {
                    Some(ResolutionStatus::Resolved) => {
                        if let Some(content) = resolved.get_instructions_rule(name) {
                            if content.is_empty() {
                                diags.push(Diagnostic::warn(
                                    "[instructions.rules]",
                                    format!("{label} file is empty"),
                                ));
                            } else if theta_static::is_rule_template(content) {
                                diags.push(Diagnostic::warn(
                                    "[instructions.rules]",
                                    format!("{label} still using scaffold template"),
                                ));
                            }
                        }
                    }
                    Some(ResolutionStatus::Deferred) => {
                        // reachability diagnostics already report deferred remote refs
                    }
                    Some(ResolutionStatus::Missing) => {
                        let src_hint = rule.src.display_compact();
                        diags.push(Diagnostic::error(
                            "[instructions.rules]",
                            format!("{label} content could not be resolved: {src_hint}"),
                        ));
                    }
                    Some(ResolutionStatus::Error) => {
                        let src_hint = rule.src.display_compact();
                        let detail = resolved.get_error(&key).unwrap_or("unknown resolver error");
                        diags.push(Diagnostic::error(
                            "[instructions.rules]",
                            format!("{label} content could not be resolved: {src_hint} ({detail})"),
                        ));
                    }
                    None => {
                        let src_hint = rule.src.display_compact();
                        diags.push(Diagnostic::error(
                            "[instructions.rules]",
                            format!("{label} content could not be resolved: {src_hint}"),
                        ));
                    }
                }
            }
        }
    }
}

impl ValidateContent for Skill {
    fn validate_content(&self, name: &str, resolved: &ResolvedRefs, diags: &mut Vec<Diagnostic>) {
        let key = ResolvedRefKey::skill(name);
        let ctx = format!("[skills.{name}]");
        match resolved.get_status(&key) {
            Some(ResolutionStatus::Resolved) => {
                if let Some(content) = resolved.get(&key) {
                    if theta_static::is_skill_template(content) {
                        diags.push(Diagnostic::warn(
                            &ctx,
                            "SKILL.md still using scaffold template - edit it",
                        ));
                        return;
                    }
                    let Some(yaml_str) = theta_static::split_frontmatter(content).0 else {
                        diags.push(Diagnostic::warn(
                            &ctx,
                            "SKILL.md has no YAML frontmatter (expected --- delimiters)",
                        ));
                        return;
                    };
                    match SkillFrontmatter::parse(yaml_str) {
                        Ok(fm) => {
                            match fm.name.as_deref() {
                                Some(n) if n != name => {
                                    diags.push(Diagnostic::error(
                                        &ctx,
                                        format!(
                                            "SKILL.md frontmatter name \"{n}\" does not match skill key \"{name}\""
                                        ),
                                    ));
                                }
                                None => {
                                    diags.push(Diagnostic::error(
                                        &ctx,
                                        "SKILL.md frontmatter is missing required `name` field",
                                    ));
                                }
                                _ => {}
                            }
                            match fm.description.as_deref() {
                                Some("") => {
                                    diags.push(Diagnostic::error(
                                        &ctx,
                                        "SKILL.md frontmatter `description` is empty",
                                    ));
                                }
                                None => {
                                    diags.push(Diagnostic::error(
                                        &ctx,
                                        "SKILL.md frontmatter is missing required `description` field",
                                    ));
                                }
                                _ => {}
                            }
                        }
                        Err(_) => {
                            diags.push(Diagnostic::warn(
                                &ctx,
                                "SKILL.md frontmatter could not be parsed as YAML",
                            ));
                        }
                    }
                }
            }
            Some(ResolutionStatus::Deferred) => {}
            Some(ResolutionStatus::Missing) => {
                diags.push(Diagnostic::error(
                    &ctx,
                    format!("skill \"{name}\" SKILL.md could not be resolved"),
                ));
            }
            Some(ResolutionStatus::Error) => {
                if let Some(err) = resolved.get_error(&key) {
                    diags.push(Diagnostic::error(&ctx, format!("skill \"{name}\": {err}")));
                }
            }
            None => {}
        }
    }
}

impl ValidateContent for ThetaManifest {
    fn validate_content(&self, _name: &str, resolved: &ResolvedRefs, diags: &mut Vec<Diagnostic>) {
        if let Some(ref instructions) = self.instructions {
            instructions.validate_content("", resolved, diags);
        }
        if let Some(ref skills) = self.skills {
            for (name, skill) in skills {
                skill.validate_content(name, resolved, diags);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApplyMode, DiagLevel, LocalOrGitRef, LocalPathRef, Rule};
    use std::collections::BTreeMap;

    #[test]
    fn validate_content_errors_when_system_content_is_missing() {
        let instructions = Instructions {
            system: Some(LocalPathRef::from("instructions/system.md")),
            rules: None,
        };
        let mut resolved = ResolvedRefs::new();
        resolved.insert_missing(ResolvedRefKey::instructions_system());
        let mut diags = Vec::new();
        instructions.validate_content("", &resolved, &mut diags);

        assert!(diags.iter().any(|d| {
            d.level == DiagLevel::Error
                && d.path == "[instructions].system"
                && d.message.contains("could not be resolved")
        }));
    }

    #[test]
    fn validate_content_errors_when_local_rule_content_is_missing() {
        let mut rules = BTreeMap::new();
        rules.insert(
            "safety".to_string(),
            Rule {
                src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/safety.md")),
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
        let mut resolved = ResolvedRefs::new();
        resolved.insert_missing(ResolvedRefKey::instructions_rule("safety"));
        let mut diags = Vec::new();
        instructions.validate_content("", &resolved, &mut diags);

        assert!(diags.iter().any(|d| {
            d.level == DiagLevel::Error
                && d.path == "[instructions.rules]"
                && d.message.contains("could not be resolved")
        }));
    }

    #[test]
    fn validate_content_skips_deferred_rule_diagnostics() {
        let mut rules = BTreeMap::new();
        rules.insert(
            "remote".to_string(),
            Rule {
                src: LocalOrGitRef::Git {
                    git: "https://example.com/repo.git".to_string(),
                    branch: Some("main".to_string()),
                    tag: None,
                    rev: None,
                    file: "rules/remote.md".to_string(),
                },
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
        let mut resolved = ResolvedRefs::new();
        resolved.insert_deferred(ResolvedRefKey::instructions_rule("remote"));

        let mut diags = Vec::new();
        instructions.validate_content("", &resolved, &mut diags);

        assert!(diags.is_empty());
    }

    // skill content validation
    fn manifest_with_skill(name: &str) -> ThetaManifest {
        let mut m = crate::minimal_manifest("test");
        m.agent.description = "desc".to_string();
        let mut skills = BTreeMap::new();
        skills.insert(
            name.to_string(),
            crate::Skill {
                source: crate::SourceRef::Path {
                    path: theta_static::ThetaProjectLayout::skill_rel(name),
                },
                tags: None,
                goal: None,
            },
        );
        m.skills = Some(skills);
        m
    }

    #[test]
    fn validate_skill_content_valid_frontmatter() {
        let manifest = manifest_with_skill("my-skill");
        let mut resolved = ResolvedRefs::new();
        resolved.insert_resolved(
            ResolvedRefKey::skill("my-skill"),
            "---\nname: my-skill\ndescription: does cool stuff\n---\n# my-skill\n".to_string(),
        );

        let mut diags = Vec::new();
        manifest.validate_content("", &resolved, &mut diags);
        assert!(
            !diags.iter().any(|d| d.level == DiagLevel::Error),
            "valid skill should not produce errors: {diags:?}"
        );
    }

    #[test]
    fn validate_skill_content_name_mismatch_is_error() {
        let manifest = manifest_with_skill("my-skill");
        let mut resolved = ResolvedRefs::new();
        resolved.insert_resolved(
            ResolvedRefKey::skill("my-skill"),
            "---\nname: wrong-name\ndescription: does stuff\n---\n".to_string(),
        );

        let mut diags = Vec::new();
        manifest.validate_content("", &resolved, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Error && d.message.contains("does not match")),
            "name mismatch should be error: {diags:?}"
        );
    }

    #[test]
    fn validate_skill_content_missing_name_is_error() {
        let manifest = manifest_with_skill("my-skill");
        let mut resolved = ResolvedRefs::new();
        resolved.insert_resolved(
            ResolvedRefKey::skill("my-skill"),
            "---\ndescription: does stuff\n---\n".to_string(),
        );

        let mut diags = Vec::new();
        manifest.validate_content("", &resolved, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Error
                    && d.message.contains("missing required `name`")),
            "missing name should be error: {diags:?}"
        );
    }

    #[test]
    fn validate_skill_content_empty_description_is_error() {
        let manifest = manifest_with_skill("my-skill");
        let mut resolved = ResolvedRefs::new();
        resolved.insert_resolved(
            ResolvedRefKey::skill("my-skill"),
            "---\nname: my-skill\ndescription: \n---\n".to_string(),
        );

        let mut diags = Vec::new();
        manifest.validate_content("", &resolved, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Error && d.message.contains("description")),
            "empty description should be error: {diags:?}"
        );
    }

    #[test]
    fn validate_skill_content_no_frontmatter_warns() {
        let manifest = manifest_with_skill("my-skill");
        let mut resolved = ResolvedRefs::new();
        resolved.insert_resolved(
            ResolvedRefKey::skill("my-skill"),
            "# just a heading\nsome content\n".to_string(),
        );

        let mut diags = Vec::new();
        manifest.validate_content("", &resolved, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Warn && d.message.contains("no YAML frontmatter")),
            "missing frontmatter should warn: {diags:?}"
        );
    }

    #[test]
    fn validate_skill_content_deferred_is_silent() {
        let manifest = manifest_with_skill("remote-skill");
        let mut resolved = ResolvedRefs::new();
        resolved.insert_deferred(ResolvedRefKey::skill("remote-skill"));

        let mut diags = Vec::new();
        manifest.validate_content("", &resolved, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn validate_skill_content_scaffold_template_warns() {
        let manifest = manifest_with_skill("new-skill");
        let mut resolved = ResolvedRefs::new();
        resolved.insert_resolved(
            ResolvedRefKey::skill("new-skill"),
            theta_static::skill_template("new-skill", theta_static::DEFAULT_SKILL_DESCRIPTION),
        );

        let mut diags = Vec::new();
        manifest.validate_content("", &resolved, &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == DiagLevel::Warn && d.message.contains("scaffold template")),
            "scaffold template should warn: {diags:?}"
        );
    }
}
