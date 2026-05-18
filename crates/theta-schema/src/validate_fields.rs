use crate::{
    Agent, ApplyMode, Diagnostic, Instructions, Rule, Skill, Subagent, ThetaManifest, Tool,
    Validate,
};

impl Validate for Agent {
    fn validate(&self, diags: &mut Vec<Diagnostic>) {
        if !theta_static::is_valid_kebab_name(&self.name) {
            diags.push(Diagnostic::error(
                "[agent].name",
                format!(
                    "\"{}\" is not a valid agent name (lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens)",
                    self.name
                ),
            ));
        }
        if theta_static::is_placeholder_description(&self.description) {
            diags.push(Diagnostic::warn(
                "[agent].description",
                "still the placeholder - edit it to describe your agent",
            ));
        }
        if self.description.len() > theta_static::MAX_DESCRIPTION_LENGTH {
            diags.push(Diagnostic::error(
                "[agent].description",
                format!(
                    "exceeds {} chars ({} chars)",
                    theta_static::MAX_DESCRIPTION_LENGTH,
                    self.description.len()
                ),
            ));
        }
        if let Some(ref version) = self.version {
            if !crate::is_strict_semver(version) {
                diags.push(Diagnostic::error(
                    "[agent].version",
                    format!("\"{version}\" is not valid semver (expected major.minor.patch)"),
                ));
            }
        }
        if let Some(ref authors) = self.authors {
            for author in authors {
                if !crate::is_valid_author(author) {
                    diags.push(Diagnostic::error(
                        "[agent].authors",
                        format!(
                            "entry \"{author}\" does not match expected format (\"Name\" or \"Name <email>\")"
                        ),
                    ));
                }
            }
        }
        if let Some(ref tags) = self.tags {
            validate_tags(diags, "[agent].tags", tags);
        }
    }
}

impl Validate for ThetaManifest {
    fn validate(&self, diags: &mut Vec<Diagnostic>) {
        self.agent.validate(diags);
        if let Some(ref instructions) = self.instructions {
            instructions.validate(diags);
        }
        if let Some(ref tools) = self.tools {
            for (name, tool) in tools {
                tool.validate_named(name, diags);
            }
        }
        if let Some(ref skills) = self.skills {
            for (name, skill) in skills {
                skill.validate_named(name, diags);
            }
        }
        if let Some(ref subagents) = self.subagents {
            let mut seen_names = std::collections::HashSet::new();
            for sub in subagents {
                if !sub.name.is_empty() && !seen_names.insert(&sub.name) {
                    diags.push(Diagnostic::error(
                        "[[subagents]]",
                        format!("duplicate subagent name \"{}\"", sub.name),
                    ));
                }
                sub.validate(diags);
            }
        }
    }
}

impl Validate for Instructions {
    fn validate(&self, diags: &mut Vec<Diagnostic>) {
        if let Some(ref system) = self.system {
            diags.extend(check_absolute_path(
                "[instructions].system",
                system.as_str(),
                "system",
            ));
            diags.extend(check_dot_theta_path(
                "[instructions].system",
                system.as_str(),
                "system",
            ));
            if !system.as_str().ends_with(".md") {
                diags.push(Diagnostic::error(
                    "[instructions].system",
                    format!(
                        "path \"{}\" does not end in .md - system prompt must be markdown",
                        system.as_str()
                    ),
                ));
            }
        }

        // rules without a system prompt is suspicious
        if self.system.is_none() {
            if let Some(ref rules) = self.rules {
                if !rules.is_empty() {
                    diags.push(Diagnostic::warn(
                        "[instructions]",
                        "rules are registered but [instructions].system is not set - consider running `theta add system`",
                    ));
                }
            }
        }

        let Some(ref rules) = self.rules else {
            return;
        };
        for (name, rule) in rules {
            if !crate::is_valid_rule_name(name) {
                diags.push(Diagnostic::error(
                    "[instructions.rules]",
                    format!(
                        "\"{name}\" is not a valid rule name (kebab-case segments separated by `/`, no leading/trailing `/`)"
                    ),
                ));
                continue; // don't validate further with a bad name
            }
            if name.contains('/') {
                let depth = name.matches('/').count() + 1;
                if depth >= 3 {
                    diags.push(Diagnostic::warn(
                        "[instructions.rules]",
                        format!(
                            "rule \"{name}\" has {depth} nesting levels - consider flattening (deeply nested rules are an anti-pattern)"
                        ),
                    ));
                }
            }
            rule.validate_named(name, diags);
        }
    }
}

impl Rule {
    pub fn validate_named(&self, name: &str, diags: &mut Vec<Diagnostic>) {
        let ctx = format!("[instructions.rules.{name}]");

        match &self.src {
            crate::LocalOrGitRef::Local(path) => {
                diags.extend(check_absolute_path(
                    "[instructions.rules]",
                    path.as_str(),
                    "src",
                ));
                diags.extend(check_dot_theta_path(
                    "[instructions.rules]",
                    path.as_str(),
                    "src",
                ));
                if !path.as_str().ends_with(".md") {
                    diags.push(Diagnostic::error(
                        "[instructions.rules]",
                        format!(
                            "{} src \"{}\" does not end in .md - rule files must be markdown",
                            ctx,
                            path.as_str()
                        ),
                    ));
                }
            }
            crate::LocalOrGitRef::Git {
                git,
                branch,
                tag,
                rev,
                ..
            } => {
                diags.extend(validate_git_url(&ctx, git));
                diags.extend(validate_ref_fields(
                    &ctx,
                    branch.as_ref(),
                    tag.as_ref(),
                    rev.as_ref(),
                ));
            }
            crate::LocalOrGitRef::System { system } => {
                if !crate::is_valid_system_store_rule_name(system) {
                    diags.push(Diagnostic::error(
                        "[instructions.rules]",
                        format!("{ctx} system store name \"{system}\" is not a valid store name "),
                    ));
                }
            }
        }

        if self.apply == ApplyMode::ModelDecision && self.description.is_none() {
            diags.push(Diagnostic::error(
                "[instructions.rules]",
                format!(
                    "{ctx} has apply = \"model-decision\" but no description - the model needs a description to decide when to apply this rule"
                ),
            ));
        }

        if self.apply != ApplyMode::Glob && self.apply_to.is_some() {
            diags.push(Diagnostic::warn(
                "[instructions.rules]",
                format!(
                    "{ctx} has apply_to patterns but apply is not \"glob\" - patterns will be ignored"
                ),
            ));
        }

        if self.apply == ApplyMode::Glob
            && self.apply_to.as_ref().is_none_or(std::vec::Vec::is_empty)
        {
            diags.push(Diagnostic::warn(
                "[instructions.rules]",
                format!(
                    "{ctx} has apply = \"glob\" but no apply_to patterns - rule will never activate"
                ),
            ));
        }
    }
}

impl Tool {
    pub fn validate_named(&self, name: &str, diags: &mut Vec<Diagnostic>) {
        let ctx = format!("[tools.{name}]");

        if !crate::is_valid_tool_name(name) {
            diags.push(Diagnostic::error(
                "[tools]",
                format!(
                    "\"{name}\" is not a valid tool name (lowercase alphanumeric + hyphens, no leading/trailing hyphens)"
                ),
            ));
        }

        // transport: exactly one of command or url
        match (&self.command, &self.url) {
            (None, None) => {
                diags.push(Diagnostic::error(
                    &ctx,
                    "tool must declare either `command` (stdio) or `url` (http)",
                ));
            }
            (Some(_), Some(_)) => {
                diags.push(Diagnostic::error(
                    &ctx,
                    "tool declares both `command` and `url` - pick one transport",
                ));
            }
            (Some(cmd), None) => {
                if cmd.is_empty() {
                    diags.push(Diagnostic::error(
                        &ctx,
                        "command array is empty - at minimum the executable path is required",
                    ));
                }
            }
            (None, Some(url)) => {
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    diags.push(Diagnostic::error(
                        &ctx,
                        format!("url \"{url}\" does not start with http:// or https://"),
                    ));
                }
            }
        }

        if let Some(ref env) = self.env {
            for key in env.keys() {
                if !is_valid_env_key(key) {
                    diags.push(Diagnostic::error(
                        &ctx,
                        format!(
                            "env key \"{key}\" is not a valid POSIX environment variable name ([A-Za-z_][A-Za-z0-9_]*)"
                        ),
                    ));
                }
            }
        }

        if let Some(ref headers) = self.headers {
            if !headers.is_empty() && self.command.is_some() {
                diags.push(Diagnostic::error(
                    &ctx,
                    "headers are set on a stdio tool - headers only apply to HTTP transport",
                ));
            }
        }
    }
}

impl Skill {
    pub fn validate_named(&self, name: &str, diags: &mut Vec<Diagnostic>) {
        let ctx = format!("[skills.{name}]");

        if !crate::is_valid_skill_name(name) {
            diags.push(Diagnostic::error(
                "[skills]",
                format!(
                    "\"{name}\" is not a valid skill name (lowercase alphanumeric + hyphens, max 64 chars)"
                ),
            ));
        }

        // source-specific structural checks
        match &self.source {
            crate::SourceRef::Path { path } => {
                if path.is_empty() {
                    diags.push(Diagnostic::error(&ctx, "source path is empty"));
                }
                diags.extend(check_absolute_path(&ctx, path, "path"));
                diags.extend(check_dot_theta_path(&ctx, path, "path"));
            }
            crate::SourceRef::Git {
                git,
                branch,
                tag,
                rev,
                ..
            } => {
                diags.extend(validate_git_url(&ctx, git));
                diags.extend(validate_ref_fields(
                    &ctx,
                    branch.as_ref(),
                    tag.as_ref(),
                    rev.as_ref(),
                ));
            }
            crate::SourceRef::System { system } => {
                if !crate::is_valid_skill_name(system) {
                    diags.push(Diagnostic::error(
                        &ctx,
                        format!("system store name \"{system}\" is not a valid skill name"),
                    ));
                }
            }
        }

        if let Some(ref tags) = self.tags {
            validate_tags(diags, &ctx, tags);
        }
        if let Some(ref goal) = self.goal {
            validate_goal(diags, &ctx, goal);
        }
    }
}

impl Validate for Subagent {
    fn validate(&self, diags: &mut Vec<Diagnostic>) {
        let ctx = format!("[[subagents.{}]]", self.name);
        if self.name.is_empty() {
            diags.push(Diagnostic::error(
                "[[subagents]]",
                "subagent has an empty name",
            ));
            return;
        }

        if !theta_static::is_valid_kebab_name(&self.name) {
            diags.push(Diagnostic::error(
                "[[subagents]]",
                format!(
                    "\"{}\" is not a valid subagent name (lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens)",
                    self.name
                ),
            ));
        }

        if self.agent_ref.is_some() && self.prompt_path.is_some() {
            diags.push(Diagnostic::error(
                &ctx,
                "`ref` and `prompt_path` are mutually exclusive - use one or the other",
            ));
        }

        if let Some(ref path) = self.agent_ref {
            let path_str = path.as_str();
            diags.extend(check_absolute_path(&ctx, path_str, "ref"));
            diags.extend(check_dot_theta_path(&ctx, path_str, "ref"));
            if !path_str.ends_with(".toml") {
                diags.push(Diagnostic::error(
                    &ctx,
                    format!(
                        "ref path \"{path_str}\" does not end in .toml - expected a theta.toml manifest"
                    ),
                ));
            }
            if self.model.is_some() {
                diags.push(Diagnostic::warn(
                    &ctx,
                    "ref subagent has `model` set - ignored, the child manifest owns the model",
                ));
            }
            if self.tools.is_some() {
                diags.push(Diagnostic::warn(
                    &ctx,
                    "ref subagent has `tools` set - ignored, the child manifest owns tool declarations",
                ));
            }
            if self.skills.is_some() {
                diags.push(Diagnostic::warn(
                    &ctx,
                    "ref subagent has `skills` set - ignored, the child manifest owns skill declarations",
                ));
            }
        }

        if let Some(ref path) = self.prompt_path {
            diags.extend(check_absolute_path(&ctx, path, "prompt_path"));
            diags.extend(check_dot_theta_path(&ctx, path, "prompt_path"));
            if !path.ends_with(".md") {
                diags.push(Diagnostic::error(
                    &ctx,
                    format!(
                        "prompt_path \"{path}\" does not end in .md - expected a markdown file"
                    ),
                ));
            }
        }

        // Non-ref subagents (inline-prompt or description-only) MUST carry a
        // description per `manifest/subagents.md` -- the parent model has no
        // other signal for when to delegate. ref-style subagents inherit the
        // description from the referenced child manifest, so the local field
        // can stay empty.
        if self.agent_ref.is_none() && self.description.as_deref().is_none_or(str::is_empty) {
            diags.push(Diagnostic::error(
                &ctx,
                "non-ref subagent must have a non-empty description - the model needs context to delegate effectively",
            ));
        }

        validate_string_list(diags, &ctx, "tools", self.tools.as_deref());
        validate_string_list(diags, &ctx, "skills", self.skills.as_deref());
    }
}

// helpers - kept near the bottom, away from the trait impls

fn check_dot_theta_path(context: &str, path: &str, field_name: &str) -> Option<Diagnostic> {
    let dt = theta_static::DOT_THETA_DIR;
    let hits_dot_theta = path == dt
        || path.starts_with(&format!("{dt}/"))
        || path.starts_with(&format!("{dt}\\"))
        || path.contains(&format!("/{dt}/"))
        || path.contains(&format!("\\{dt}\\"));
    hits_dot_theta.then(|| {
        Diagnostic::error(
            context,
            format!(
                "path \"{path}\" in `{field_name}` references .theta/ - use the source file, not the materialized output"
            ),
        )
    })
}

fn check_absolute_path(context: &str, path: &str, field_name: &str) -> Option<Diagnostic> {
    std::path::Path::new(path).is_absolute().then(|| {
        Diagnostic::error(
            context,
            format!(
                "path \"{path}\" in `{field_name}` is absolute - all paths must be relative to theta.toml"
            ),
        )
    })
}

fn validate_string_list(
    diags: &mut Vec<Diagnostic>,
    ctx: &str,
    field: &str,
    list: Option<&[String]>,
) {
    if let Some(items) = list {
        if items.iter().any(std::string::String::is_empty) {
            diags.push(Diagnostic::error(
                ctx,
                format!("{field} list contains an empty string"),
            ));
        }
    }
}

fn validate_goal(diags: &mut Vec<Diagnostic>, ctx: &str, goal: &str) {
    if goal.len() > theta_static::MAX_GOAL_LENGTH {
        diags.push(Diagnostic::error(
            ctx,
            format!(
                "goal exceeds {} chars ({} chars)",
                theta_static::MAX_GOAL_LENGTH,
                goal.len()
            ),
        ));
    }
}

fn validate_tags(diags: &mut Vec<Diagnostic>, ctx: &str, tags: &[String]) {
    for tag in tags {
        if tag.is_empty() {
            diags.push(Diagnostic::error(ctx, "tags list contains an empty string"));
        } else if tag.len() > 64 {
            diags.push(Diagnostic::error(
                ctx,
                format!("tag \"{tag}\" exceeds 64 characters"),
            ));
        } else if !theta_static::is_valid_kebab_name(tag) {
            diags.push(Diagnostic::error(
                ctx,
                format!(
                    "tag \"{tag}\" is not valid (lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens)"
                ),
            ));
        }
    }
}

fn validate_ref_fields(
    ctx: &str,
    branch: Option<&String>,
    tag: Option<&String>,
    rev: Option<&String>,
) -> Vec<Diagnostic> {
    let count = [&branch, &tag, &rev].iter().filter(|f| f.is_some()).count();
    if count > 1 {
        return vec![Diagnostic::error(
            ctx,
            "at most one of `branch`, `tag`, or `rev` may be set".to_string(),
        )];
    }
    Vec::new()
}

fn validate_git_url(ctx: &str, url: &str) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    if url.contains('@') && url.contains(':') && !url.contains("://") {
        out.push(Diagnostic::error(
            ctx,
            format!(
                "git url \"{url}\" looks like SCP syntax - use ssh:// URL form instead (e.g. ssh://git@github.com/org/repo)"
            ),
        ));
        return out;
    }
    if !url.starts_with("https://")
        && !url.starts_with("http://")
        && !url.starts_with("git://")
        && !url.starts_with("ssh://")
    {
        out.push(Diagnostic::error(
            ctx,
            format!(
                "git url \"{url}\" has an unrecognized scheme - supported: https://, http://, git://, ssh://"
            ),
        ));
        return out;
    }
    if let Some(scheme_end) = url.find("://") {
        let authority = &url[scheme_end + 3..];
        if authority.contains('@') && !authority.starts_with("git@") {
            out.push(Diagnostic::warn(
                ctx,
                format!(
                    "git url \"{url}\" contains embedded credentials - use a credential helper instead"
                ),
            ));
        }
    }
    out
}

fn is_valid_env_key(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.starts_with(|c: char| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LocalOrGitRef, LocalPathRef, minimal_manifest};

    #[test]
    fn validate_named_does_not_reject_git_rule_without_md_suffix() {
        let rule = Rule {
            src: LocalOrGitRef::Git {
                git: "https://example.com/repo.git".to_string(),
                branch: Some("main".to_string()),
                tag: None,
                rev: None,
                file: "rulefile".to_string(),
            },
            summary: None,
            description: None,
            apply: ApplyMode::Always,
            apply_to: None,
        };
        let mut diags = Vec::new();
        rule.validate_named("git-rule", &mut diags);

        assert!(!diags.iter().any(|d| d.level == crate::DiagLevel::Error));
    }

    #[test]
    fn validate_fresh_manifest_has_placeholder_warnings() {
        let manifest = minimal_manifest("test");
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert_eq!(diags.len(), 1);
        assert!(diags.iter().any(|d| d.path == "[agent].description"));
    }

    #[test]
    fn validate_clean_manifest_has_no_warnings() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "A real agent description".to_string();
        manifest.agent.model = Some("claude-sonnet-4-20250514".to_string());
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn validate_rules_with_bad_name_errors() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        manifest.agent.model = Some("claude-sonnet-4-20250514".to_string());
        let mut rules = std::collections::BTreeMap::new();
        rules.insert(
            "Bad_Name".to_string(),
            Rule {
                src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/bad.md")),
                summary: None,
                description: None,
                apply: ApplyMode::Always,
                apply_to: None,
            },
        );
        manifest.instructions = Some(Instructions {
            system: None,
            rules: Some(rules),
        });

        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.path == "[instructions.rules]")
        );
    }

    fn make_subagent(name: &str) -> Subagent {
        Subagent {
            name: name.to_string(),
            description: Some("test subagent".to_string()),
            agent_ref: None,
            prompt_path: None,
            model: None,
            tools: None,
            skills: None,
        }
    }

    #[test]
    fn validate_subagent_ref_and_prompt_path_mutual_exclusion() {
        let mut sub = make_subagent("both");
        sub.agent_ref = Some("agents/child.toml".into());
        sub.prompt_path = Some("subagents/both.md".to_string());
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error
                    && d.message.contains("mutually exclusive")),
            "expected mutual-exclusion error, got: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_prompt_path_errors_without_md() {
        let mut sub = make_subagent("no-ext");
        sub.prompt_path = Some("subagents/no-ext.txt".to_string());
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains(".md")),
            "expected .md error, got: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_prompt_path_errors_under_dot_theta() {
        let mut sub = make_subagent("bad-path");
        sub.prompt_path = Some(".theta/subagents/bad.md".to_string());
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains(".theta/")),
            "expected .theta/ error, got: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_description_only_is_clean() {
        let sub = make_subagent("helper");
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        // no errors (a description-only subagent with description set is valid)
        assert!(
            !diags.iter().any(|d| d.level == crate::DiagLevel::Error),
            "unexpected errors: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_missing_description_is_error() {
        let mut sub = make_subagent("helper");
        sub.description = None;
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            diags.iter().any(|d| d.level == crate::DiagLevel::Error
                && d.message
                    .contains("non-ref subagent must have a non-empty description")),
            "expected missing-description error, got: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_prompt_path_valid_is_clean() {
        let mut sub = make_subagent("good");
        sub.prompt_path = Some("subagents/good.md".to_string());
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            !diags.iter().any(|d| d.level == crate::DiagLevel::Error),
            "unexpected errors: {diags:?}"
        );
        assert!(
            !diags.iter().any(|d| d.level == crate::DiagLevel::Warn),
            "unexpected warnings: {diags:?}"
        );
    }

    #[test]
    fn validate_git_url_accepts_supported_schemes() {
        for url in [
            "https://github.com/org/repo",
            "http://github.com/org/repo",
            "git://github.com/org/repo",
            "ssh://git@github.com/org/repo",
        ] {
            assert!(
                validate_git_url("[test]", url).is_empty(),
                "expected {url} to be accepted"
            );
        }
    }

    #[test]
    fn validate_git_url_rejects_scp_with_helpful_message() {
        let diags = validate_git_url("[test]", "git@github.com:org/repo.git");
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("SCP syntax"),
            "expected SCP hint, got: {}",
            diags[0].message
        );
        assert!(
            diags[0].message.contains("ssh://"),
            "expected ssh:// suggestion, got: {}",
            diags[0].message
        );
    }

    #[test]
    fn validate_git_url_rejects_unknown_scheme() {
        let diags = validate_git_url("[test]", "ftp://example.com/repo");
        assert_eq!(diags.len(), 1);
        assert!(
            diags[0].message.contains("unrecognized scheme"),
            "expected scheme error, got: {}",
            diags[0].message
        );
    }

    #[test]
    fn validate_git_url_rejects_bare_path() {
        let diags = validate_git_url("[test]", "/some/local/path");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("unrecognized scheme"));
    }

    #[test]
    fn rule_with_scp_git_url_produces_error() {
        let rule = Rule {
            src: LocalOrGitRef::Git {
                git: "git@github.com:org/rules.git".to_string(),
                branch: None,
                tag: None,
                rev: None,
                file: "safety.md".to_string(),
            },
            summary: None,
            description: None,
            apply: crate::ApplyMode::Always,
            apply_to: None,
        };
        let mut diags = Vec::new();
        rule.validate_named("scp-rule", &mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("SCP")),
            "expected SCP error for rule, got: {diags:?}"
        );
    }

    #[test]
    fn validate_ref_fields_allows_single_branch() {
        let diags = validate_ref_fields("[test]", Some(&"main".into()), None, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn validate_ref_fields_allows_single_tag() {
        let diags = validate_ref_fields("[test]", None, Some(&"v1.0".into()), None);
        assert!(diags.is_empty());
    }

    #[test]
    fn validate_ref_fields_allows_none() {
        let diags = validate_ref_fields("[test]", None, None, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn validate_ref_fields_rejects_branch_and_tag() {
        let diags = validate_ref_fields("[test]", Some(&"main".into()), Some(&"v1.0".into()), None);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("at most one"));
    }

    #[test]
    fn git_skill_with_explicit_branch_deserializes() {
        let toml_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test"

[skills.my-skill]
source = { git = "https://github.com/org/repo", branch = "main" }
"#;
        let manifest: crate::ThetaManifest = toml::from_str(toml_str).unwrap();
        let skill = manifest.skills.unwrap();
        let s = &skill["my-skill"];
        match &s.source {
            crate::SourceRef::Git {
                branch, tag, rev, ..
            } => {
                assert_eq!(branch.as_deref(), Some("main"));
                assert!(tag.is_none());
                assert!(rev.is_none());
            }
            other => panic!("expected Git source, got: {other:?}"),
        }
    }

    #[test]
    fn git_skill_with_explicit_tag_deserializes() {
        let toml_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test"

[skills.my-skill]
source = { git = "https://github.com/org/repo", tag = "v2.0" }
"#;
        let manifest: crate::ThetaManifest = toml::from_str(toml_str).unwrap();
        let skill = manifest.skills.unwrap();
        let s = &skill["my-skill"];
        match &s.source {
            crate::SourceRef::Git { tag, branch, .. } => {
                assert_eq!(tag.as_deref(), Some("v2.0"));
                assert!(branch.is_none());
            }
            other => panic!("expected Git source, got: {other:?}"),
        }
    }

    // agent.name validation

    #[test]
    fn validate_agent_name_valid_kebab() {
        let mut manifest = minimal_manifest("my-agent");
        manifest.agent.description = "A valid agent".to_string();
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            !diags
                .iter()
                .any(|d| d.path == "[agent].name" && d.level == crate::DiagLevel::Error),
            "valid name should not produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_agent_name_rejects_uppercase() {
        let mut manifest = minimal_manifest("MyAgent");
        manifest.agent.description = "desc".to_string();
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].name" && d.level == crate::DiagLevel::Error),
            "uppercase name should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_agent_name_rejects_underscores() {
        let mut manifest = minimal_manifest("my_agent");
        manifest.agent.description = "desc".to_string();
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].name" && d.level == crate::DiagLevel::Error),
            "underscore name should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_agent_name_rejects_empty() {
        let mut manifest = minimal_manifest("");
        manifest.agent.description = "desc".to_string();
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].name" && d.level == crate::DiagLevel::Error),
            "empty name should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_agent_name_rejects_leading_hyphen() {
        let mut manifest = minimal_manifest("-leading");
        manifest.agent.description = "desc".to_string();
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].name" && d.level == crate::DiagLevel::Error),
        );
    }

    #[test]
    fn validate_agent_name_rejects_consecutive_hyphens() {
        let mut manifest = minimal_manifest("bad--name");
        manifest.agent.description = "desc".to_string();
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].name" && d.level == crate::DiagLevel::Error),
        );
    }

    #[test]
    fn validate_agent_name_accepts_digits() {
        let mut manifest = minimal_manifest("agent-v2");
        manifest.agent.description = "desc".to_string();
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            !diags
                .iter()
                .any(|d| d.path == "[agent].name" && d.level == crate::DiagLevel::Error),
            "alphanumeric kebab should be valid: {diags:?}"
        );
    }

    // subagent.name validation

    #[test]
    fn validate_subagent_name_rejects_uppercase() {
        let sub = make_subagent("BadName");
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            diags.iter().any(|d| d.level == crate::DiagLevel::Error
                && d.message.contains("not a valid subagent name")),
            "uppercase subagent name should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_name_rejects_underscores() {
        let sub = make_subagent("bad_name");
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(diags.iter().any(|d| d.level == crate::DiagLevel::Error
            && d.message.contains("not a valid subagent name")),);
    }

    #[test]
    fn validate_subagent_name_accepts_valid_kebab() {
        let sub = make_subagent("my-researcher");
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            !diags.iter().any(|d| d.level == crate::DiagLevel::Error),
            "valid kebab subagent name should not produce error: {diags:?}"
        );
    }

    // severity upgrades

    #[test]
    fn validate_description_over_limit_is_error() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "x".repeat(theta_static::MAX_DESCRIPTION_LENGTH + 1);
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].description" && d.level == crate::DiagLevel::Error),
            "description over limit should be error: {diags:?}"
        );
    }

    #[test]
    fn validate_version_not_semver_is_error() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        manifest.agent.version = Some("not-semver".to_string());
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].version" && d.level == crate::DiagLevel::Error),
            "bad version should be error: {diags:?}"
        );
    }

    #[test]
    fn validate_authors_bad_format_is_error() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        manifest.agent.authors = Some(vec!["Name <broken-email".to_string()]);
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.path == "[agent].authors" && d.level == crate::DiagLevel::Error),
            "bad author format should be error: {diags:?}"
        );
    }

    #[test]
    fn validate_model_decision_without_description_is_error() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        let mut rules = std::collections::BTreeMap::new();
        rules.insert(
            "conditional".to_string(),
            Rule {
                src: LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/cond.md")),
                summary: None,
                description: None,
                apply: ApplyMode::ModelDecision,
                apply_to: None,
            },
        );
        manifest.instructions = Some(Instructions {
            system: Some(LocalPathRef::from("instructions/system.md")),
            rules: Some(rules),
        });
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags.iter().any(|d| d.level == crate::DiagLevel::Error
                && d.message.contains("model-decision")),
            "model-decision without description should be error: {diags:?}"
        );
    }

    // tags + goal

    #[test]
    fn skill_with_tags_and_goal_deserializes() {
        let toml_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test"

[skills.deploy]
source = { path = "skills/deploy" }
tags = ["ci-cd", "cloud", "deployment"]
goal = "Deploy application to production"
"#;
        let manifest: crate::ThetaManifest = toml::from_str(toml_str).unwrap();
        let skill = &manifest.skills.unwrap()["deploy"];
        assert_eq!(
            skill.tags.as_deref().unwrap(),
            &["ci-cd", "cloud", "deployment"]
        );
        assert_eq!(
            skill.goal.as_deref().unwrap(),
            "Deploy application to production"
        );
    }

    #[test]
    fn skill_without_tags_and_goal_still_works() {
        let toml_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "test"
description = "test"

[skills.deploy]
source = { path = "skills/deploy" }
"#;
        let manifest: crate::ThetaManifest = toml::from_str(toml_str).unwrap();
        let skill = &manifest.skills.unwrap()["deploy"];
        assert!(skill.tags.is_none());
        assert!(skill.goal.is_none());
    }

    #[test]
    fn skill_tag_rejects_uppercase() {
        let mut manifest = minimal_manifest("test");
        let mut skills = std::collections::BTreeMap::new();
        skills.insert(
            "deploy".to_string(),
            crate::Skill {
                source: crate::SourceRef::Path {
                    path: "skills/deploy".to_string(),
                },
                tags: Some(vec!["CI-CD".to_string()]),
                goal: None,
            },
        );
        manifest.skills = Some(skills);
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("CI-CD")),
            "uppercase tag should be rejected: {diags:?}"
        );
    }

    #[test]
    fn skill_goal_rejects_too_long() {
        let mut manifest = minimal_manifest("test");
        let mut skills = std::collections::BTreeMap::new();
        skills.insert(
            "deploy".to_string(),
            crate::Skill {
                source: crate::SourceRef::Path {
                    path: "skills/deploy".to_string(),
                },
                tags: None,
                goal: Some("x".repeat(513)),
            },
        );
        manifest.skills = Some(skills);
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("goal exceeds")),
            "too-long goal should be rejected: {diags:?}"
        );
    }

    #[test]
    fn agent_with_tags_deserializes() {
        let toml_str = r#"
[theta]
schema = "2026-04"

[agent]
name = "my-agent"
description = "test agent"
tags = ["python", "code-review", "backend"]
"#;
        let manifest: crate::ThetaManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(
            manifest.agent.tags.as_deref().unwrap(),
            &["python", "code-review", "backend"]
        );
    }

    #[test]
    fn agent_tag_rejects_invalid() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.tags = Some(vec!["valid".to_string(), "NOT_VALID".to_string()]);
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("NOT_VALID")),
            "invalid agent tag should be rejected: {diags:?}"
        );
    }

    // absolute path rejection

    #[test]
    fn validate_system_rejects_absolute_path() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        manifest.instructions = Some(Instructions {
            system: Some(LocalPathRef::from("/etc/system.md")),
            rules: None,
        });
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("absolute")),
            "absolute system path should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_rule_src_rejects_absolute_path() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        let mut rules = std::collections::BTreeMap::new();
        rules.insert(
            "safety".to_string(),
            Rule {
                src: LocalOrGitRef::Local(LocalPathRef::from("/absolute/rules/safety.md")),
                summary: None,
                description: None,
                apply: ApplyMode::Always,
                apply_to: None,
            },
        );
        manifest.instructions = Some(Instructions {
            system: None,
            rules: Some(rules),
        });
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("absolute")),
            "absolute rule src should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_skill_path_rejects_absolute() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        let mut skills = std::collections::BTreeMap::new();
        skills.insert(
            "my-skill".to_string(),
            Skill {
                source: crate::SourceRef::Path {
                    path: "/absolute/skills/my-skill".to_string(),
                },
                tags: None,
                goal: None,
            },
        );
        manifest.skills = Some(skills);
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("absolute")),
            "absolute skill path should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_ref_rejects_absolute() {
        let mut sub = make_subagent("child");
        sub.agent_ref = Some("/absolute/agents/child.toml".into());
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("absolute")),
            "absolute subagent ref should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_subagent_prompt_path_rejects_absolute() {
        let mut sub = make_subagent("writer");
        sub.prompt_path = Some("/absolute/subagents/writer.md".to_string());
        let mut diags = Vec::new();
        sub.validate(&mut diags);
        assert!(
            diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("absolute")),
            "absolute prompt_path should produce error: {diags:?}"
        );
    }

    #[test]
    fn validate_relative_paths_are_accepted() {
        let mut manifest = minimal_manifest("test");
        manifest.agent.description = "desc".to_string();
        manifest.instructions = Some(Instructions {
            system: Some(LocalPathRef::from("instructions/system.md")),
            rules: None,
        });
        let mut diags = Vec::new();
        manifest.validate(&mut diags);
        assert!(
            !diags
                .iter()
                .any(|d| d.level == crate::DiagLevel::Error && d.message.contains("absolute")),
            "relative path should not trigger absolute error: {diags:?}"
        );
    }

    #[test]
    fn validate_unix_absolute_path_detected() {
        assert!(
            check_absolute_path("[test]", "/usr/local/rules/safety.md", "src").is_some(),
            "unix absolute path should be rejected"
        );
    }
}
