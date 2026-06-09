//! `theta.toml` Rust types — `Serialize`, `Deserialize`, `JsonSchema`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod command_output;
mod resolved;
mod validate_content;
mod validate_fields;
use std::fmt::Write;

pub use command_output::{CommandOutput, CommandStatus};
pub use resolved::{ResolutionStatus, ResolvedRefKey, ResolvedRefs};

/// Validate a rule name: either simple kebab (`safety`) or path-qualified (`backend/typescript`).
/// Each `/`-separated segment must be valid kebab-case individually.
pub fn is_valid_rule_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.split('/').all(theta_static::is_valid_kebab_name)
}

// proxies to valid kebab name for now; it may change if the system store
// starts accepting path-qualified names in the future
pub fn is_valid_system_store_rule_name(name: &str) -> bool {
    theta_static::is_valid_kebab_name(name)
}

pub fn is_valid_tool_name(name: &str) -> bool {
    theta_static::is_valid_kebab_name(name)
}

pub fn is_valid_skill_name(name: &str) -> bool {
    if name.len() > 64 {
        return false;
    }
    if name.contains("--") {
        return false;
    }
    theta_static::is_valid_kebab_name(name)
}

/// The full `theta.toml` manifest.
///
/// Construct via deserialization (`toml::from_str`) followed by `Validate::validate`,
/// or via `minimal_manifest()` for test fixtures. Direct struct-literal construction
/// bypasses validation — callers are responsible for ensuring invariants.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ThetaManifest {
    pub theta: Theta,
    pub agent: Agent,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Instructions>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<std::collections::BTreeMap<String, Tool>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<std::collections::BTreeMap<String, Skill>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subagents: Option<Vec<Subagent>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<std::collections::BTreeMap<String, serde_json::Value>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extras: Option<std::collections::BTreeMap<String, serde_json::Value>>,
}

impl ThetaManifest {
    pub fn harness_config<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<Option<T>, serde_json::Error> {
        let Some(ref harness_map) = self.harness else {
            return Ok(None);
        };
        let Some(value) = harness_map.get(name) else {
            return Ok(None);
        };
        serde_json::from_value(value.clone()).map(Some)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Theta {
    pub schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Instructions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<LocalPathRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<BTreeMap<String, Rule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub src: LocalOrGitRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub apply: ApplyMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apply_to: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(transparent)]
pub struct LocalPathRef(pub String);

impl LocalPathRef {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for LocalPathRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for LocalPathRef {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for LocalPathRef {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum LocalOrGitRef {
    Local(LocalPathRef),
    Git {
        git: String,
        /// Explicit branch name
        #[serde(default, skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        /// Explicit tag name
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        /// Explicit commit SHA
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
        file: String,
    },
    System {
        system: String,
    },
}

impl LocalOrGitRef {
    pub fn local_path(&self) -> Option<&str> {
        match self {
            Self::Local(path) => Some(path.as_str()),
            Self::Git { .. } | Self::System { .. } => None,
        }
    }

    pub fn display_compact(&self) -> String {
        match self {
            Self::Local(path) => path.as_str().to_string(),
            Self::Git {
                git,
                branch,
                tag,
                rev,
                file,
            } => {
                let r = branch.as_deref().or(tag.as_deref()).or(rev.as_deref());
                match r {
                    Some(r) => format!("git:{git}#{r}:{file}"),
                    None => format!("git:{git}:{file}"),
                }
            }
            Self::System { system } => format!("system:{system}"),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ApplyMode {
    #[default]
    Always,
    ModelDecision,
    Glob,
    Manual,
}

impl std::fmt::Display for ApplyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ApplyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "always" => Ok(Self::Always),
            "model-decision" => Ok(Self::ModelDecision),
            "glob" => Ok(Self::Glob),
            "manual" => Ok(Self::Manual),
            other => Err(format!(
                "unknown apply mode \"{other}\": expected always, model-decision, glob, or manual"
            )),
        }
    }
}

impl ApplyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::ModelDecision => "model-decision",
            Self::Glob => "glob",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub enabled: bool,
}

impl Tool {
    pub fn transport(&self) -> &'static str {
        if self.command.is_some() {
            "stdio"
        } else if self.url.is_some() {
            "http"
        } else {
            "unknown"
        }
    }

    pub fn target(&self) -> String {
        self.command
            .as_ref()
            .map(|c| c.join(" "))
            .or_else(|| self.url.clone())
            .unwrap_or_default()
    }
}

fn default_true() -> bool {
    true
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde skip_serializing_if requires &bool
fn is_true(v: &bool) -> bool {
    *v
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    pub source: SourceRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SourceRef {
    Git {
        git: String,
        /// Explicit branch name
        #[serde(default, skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        /// Explicit tag name
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        /// Explicit commit SHA
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subdirectory: Option<String>,
    },
    Path {
        path: String,
    },
    System {
        system: String,
    },
}

impl SourceRef {
    pub fn display_compact(&self) -> (&'static str, String) {
        match self {
            Self::Path { path } => ("path", path.clone()),
            Self::Git {
                git,
                branch,
                tag,
                rev,
                subdirectory,
                ..
            } => {
                let mut s = git.clone();
                let r = branch.as_deref().or(tag.as_deref()).or(rev.as_deref());
                if let Some(r) = r {
                    let _ = write!(s, "#{r}");
                }
                if let Some(sub) = subdirectory {
                    let _ = write!(s, ":{sub}");
                }
                ("git", s)
            }
            Self::System { system } => ("system", system.clone()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Subagent {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ref")]
    pub agent_ref: Option<LocalPathRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SubagentMode {
    /// `ref` is set — a full child agent with its own manifest
    Ref,
    /// `prompt_path` is set — system prompt in a local `.md` file
    Inline,
    /// Neither `ref` nor `prompt_path` — runs with `description` only
    DescriptionOnly,
}

impl std::fmt::Display for SubagentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ref => f.write_str("ref"),
            Self::Inline => f.write_str("inline"),
            Self::DescriptionOnly => f.write_str("description-only"),
        }
    }
}

impl Subagent {
    pub fn mode(&self) -> SubagentMode {
        if self.agent_ref.is_some() {
            SubagentMode::Ref
        } else if self.prompt_path.is_some() {
            SubagentMode::Inline
        } else {
            SubagentMode::DescriptionOnly
        }
    }
}

pub fn minimal_manifest(name: &str) -> ThetaManifest {
    ThetaManifest {
        theta: Theta {
            schema: theta_static::SCHEMA_VERSION.to_string(),
        },
        agent: Agent {
            name: name.to_string(),
            description: theta_static::DEFAULT_DESCRIPTION.to_string(),
            version: Some(theta_static::DEFAULT_VERSION.to_string()),
            authors: None,
            model: None,
            tags: None,
        },
        instructions: None,
        tools: None,
        skills: None,
        subagents: None,
        harness: None,
        extras: None,
    }
}

pub fn normalize_agent_name(raw: &str) -> String {
    let normalized: String = raw
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();

    let collapsed: String = normalized
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if collapsed.is_empty() {
        "my-agent".to_string()
    } else {
        collapsed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DiagLevel {
    Error,
    Warn,
    Hint,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub path: String,
    pub message: String,
}

impl Diagnostic {
    pub fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: DiagLevel::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn warn(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: DiagLevel::Warn,
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn hint(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: DiagLevel::Hint,
            path: path.into(),
            message: message.into(),
        }
    }
}

pub trait Validate {
    fn validate(&self, diagnostics: &mut Vec<Diagnostic>);
}

pub trait ValidateContent {
    fn validate_content(&self, name: &str, resolved: &ResolvedRefs, diags: &mut Vec<Diagnostic>);
}

#[derive(Debug, serde::Deserialize)]
pub struct SkillFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl SkillFrontmatter {
    pub fn parse(yaml_str: &str) -> Result<Self, serde_norway::Error> {
        serde_norway::from_str(yaml_str)
    }
}

/// Strict semver: exactly `major.minor.patch`, no pre-release or build metadata.
fn is_strict_semver(version: &str) -> bool {
    semver::Version::parse(version)
        .ok()
        .is_some_and(|v| v.pre.is_empty() && v.build.is_empty())
}

/// Author format: non-empty, either "Name" or "Name <email>".
fn is_valid_author(author: &str) -> bool {
    let author = author.trim();
    if author.is_empty() {
        return false;
    }
    if let Some(bracket_start) = author.find('<') {
        if !author.ends_with('>') {
            return false;
        }
        let name = author[..bracket_start].trim();
        let email = &author[bracket_start + 1..author.len() - 1];
        !name.is_empty() && !email.is_empty() && email.contains('@')
    } else {
        true
    }
}

pub fn known_sections() -> Vec<String> {
    let schema = schemars::schema_for!(ThetaManifest);
    schema
        .schema
        .object
        .as_ref()
        .map(|obj| obj.properties.keys().cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_sections_matches_struct_fields() {
        let sections = known_sections();
        assert!(sections.contains(&"theta".to_string()));
        assert!(sections.contains(&"agent".to_string()));
        assert!(sections.contains(&"harness".to_string()));
        assert!(sections.contains(&"extras".to_string()));
        assert_eq!(sections.len(), 8);
    }

    #[test]
    fn normalize_agent_name_simple() {
        assert_eq!(normalize_agent_name("my-project"), "my-project");
    }

    #[test]
    fn normalize_agent_name_spaces_and_caps() {
        assert_eq!(normalize_agent_name("My Cool Agent"), "my-cool-agent");
    }

    #[test]
    fn normalize_agent_name_underscores() {
        assert_eq!(normalize_agent_name("code_reviewer"), "code-reviewer");
    }

    #[test]
    fn normalize_agent_name_empty_fallback() {
        assert_eq!(normalize_agent_name(""), "my-agent");
    }

    #[test]
    fn strict_semver_valid() {
        assert!(is_strict_semver("0.1.0"));
        assert!(is_strict_semver("1.0.0"));
        assert!(is_strict_semver("12.34.56"));
    }

    #[test]
    fn strict_semver_invalid() {
        assert!(!is_strict_semver("1.0.0-beta"));
        assert!(!is_strict_semver("1.0"));
        assert!(!is_strict_semver("1.0.0.0"));
        assert!(!is_strict_semver("v1.0.0"));
    }

    #[test]
    fn valid_author_formats() {
        assert!(is_valid_author("Alice"));
        assert!(is_valid_author("Alice <alice@example.com>"));
    }

    #[test]
    fn invalid_author_formats() {
        assert!(!is_valid_author(""));
        assert!(!is_valid_author("<broken>"));
        assert!(!is_valid_author("Alice <no-at-sign>"));
    }

    #[test]
    fn local_path_ref_as_str() {
        let parsed = LocalPathRef::from("instructions/system.md");
        assert_eq!(parsed.as_str(), "instructions/system.md");
    }

    #[test]
    fn local_or_git_ref_local_path_access() {
        let local = LocalOrGitRef::Local(LocalPathRef::from("instructions/rules/safety.md"));
        let git = LocalOrGitRef::Git {
            git: "https://example.com/repo.git".to_string(),
            branch: Some("main".to_string()),
            tag: None,
            rev: None,
            file: "rules/safety.md".to_string(),
        };

        assert_eq!(local.local_path(), Some("instructions/rules/safety.md"));
        assert_eq!(git.local_path(), None);
    }
}
