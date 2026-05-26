use anyhow::{Context, Result};
use heck::ToKebabCase;
use std::path::{Path, PathBuf};

/// Current schema version (calendar-based, bumped on breaking changes only).
pub const SCHEMA_VERSION: &str = "2026-04";

/// All schema versions, oldest first.
pub const SCHEMA_VERSIONS: &[&str] = &["2026-04"];

/// Sentinel placeholder for description.
pub const DEFAULT_DESCRIPTION: &str = "add your description here";

pub fn is_placeholder_description(description: &str) -> bool {
    description == DEFAULT_DESCRIPTION
}

/// Default version emitted by `theta init`.
pub const DEFAULT_VERSION: &str = "0.1.0";

/// Maximum description length in characters.
pub const MAX_DESCRIPTION_LENGTH: usize = 1024;

/// Maximum goal length in characters.
pub const MAX_GOAL_LENGTH: usize = 512;

/// Manifest filename.
pub const MANIFEST_FILE_NAME: &str = "theta.toml";

/// Returns true when the path points to a root `theta.toml`.
pub fn is_default_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == MANIFEST_FILE_NAME)
}

/// Lockfile filename.
pub const LOCKFILE: &str = "theta.lock";

/// Materialization directory (inside the project).
pub const DOT_THETA_DIR: &str = ".theta";

/// Default instructions directory relative to manifest.
pub const INSTRUCTIONS_DIR: &str = "instructions";

/// Default rules subdirectory within instructions.
pub const RULES_DIR: &str = "rules";

/// Reason code for git sources that need `theta sync` to resolve.
pub const GIT_UNRESOLVED_CODE: &str = "git source not yet resolved";

/// Program name for CLI output.
pub const PROGRAM_NAME: &str = "theta";

/// Directory name for subagent prompt files in the project (convention).
pub const SUBAGENTS_DIR_NAME: &str = "subagents";

/// Environment variable to override the subagent prompts directory.
///
/// When set, `theta cast from` and `ImportOptions` write externalized
/// subagent prompt `.md` files into this directory instead of the
/// default `<project>/subagents/`.
pub const THETA_SUBAGENTS_DIR_ENV: &str = "THETA_SUBAGENTS_DIR";

/// Override the theta data directory (system store, etc.).
/// Defaults to `$XDG_DATA_HOME/theta` when unset.
pub const THETA_DATA_DIR_ENV: &str = "THETA_DATA_DIR";

/// theta-typed keys for a `[tools.<name>]` MCP server entry.
pub const THETA_TYPED_MCP_KEYS: &[&str] = &[
    MCP_KEY_TYPE,
    MCP_KEY_COMMAND,
    MCP_KEY_ARGS,
    MCP_KEY_ENV,
    MCP_KEY_URL,
    MCP_KEY_HEADERS,
];

/// theta-typed keys for a `[[subagents]]` entry's frontmatter.
pub const THETA_TYPED_AGENT_KEYS: &[&str] = &["name", "description", "model", "tools"];

// MCP server-config key names, as they appear on disk in a harness MCP file
// (e.g. `.mcp.json` for claude, `.vscode/mcp.json` for copilot, `.cursor/mcp.json`
// for cursor). collected here so each harness adapter quotes the same literals.
pub const MCP_KEY_TYPE: &str = "type";
pub const MCP_KEY_COMMAND: &str = "command";
pub const MCP_KEY_ARGS: &str = "args";
pub const MCP_KEY_ENV: &str = "env";
pub const MCP_KEY_URL: &str = "url";
pub const MCP_KEY_HEADERS: &str = "headers";

/// MCP transport identifier, as written to the `type` key of a server config.
///
/// The `Http` variant additionally accepts `streamable-http` as an alias on
/// parse, per the MCP spec which renamed the http transport.
///
/// ref: <https://modelcontextprotocol.io/specification/2025-03-26/basic/transports>
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumString, strum::IntoStaticStr,
)]
#[strum(serialize_all = "lowercase")]
#[non_exhaustive]
pub enum McpTransport {
    Stdio,
    #[strum(serialize = "http", serialize = "streamable-http")]
    Http,
    Sse,
}

impl McpTransport {
    /// Canonical wire string written to the `type` key.
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

/// Sub-key under `[harness.<name>]` for per-MCP-server extras.
/// Used as `[harness.<harness>.tool.<server>]`. Part of theta's manifest
/// convention for every harness; not harness-specific.
pub const HARNESS_EXTRAS_TOOL_KEY: &str = "tool";

/// Sub-key under `[harness.<name>]` for per-subagent extras.
/// Used as `[harness.<harness>.subagent.<slug>]`. Part of theta's manifest
/// convention for every harness; not harness-specific.
pub const HARNESS_EXTRAS_SUBAGENT_KEY: &str = "subagent";

/// Rule template — `theta add rule` creates this content.
/// `theta check` warns when a rule file still matches.
pub const DEFAULT_RULE_TEMPLATE: &str = "\
# TODO: define your rule here\n\
\n\
replace this with the rule content your agent should follow.\n";

pub fn is_rule_template(content: &str) -> bool {
    content.trim() == DEFAULT_RULE_TEMPLATE.trim()
}

/// System prompt template.
pub const DEFAULT_SYSTEM_TEMPLATE: &str = "\
# TODO: define your system prompt here\n\
\n\
this is the base system prompt for your agent, describe:\n\
- what your agent does\n\
- how it should behave\n\
- any constraints or guidelines\n";

pub fn is_system_template(content: &str) -> bool {
    content.trim() == DEFAULT_SYSTEM_TEMPLATE.trim()
}

/// Base predicate: lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens.
pub fn is_valid_kebab_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let bytes = name.as_bytes();
    if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
        return false;
    }
    if name.contains("--") {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Validate a `KEY=VALUE` env pair: POSIX env name (`[A-Za-z_][A-Za-z0-9_]*`), non-empty value.
pub fn validate_env_pair(pair: &str) -> Result<(&str, &str), &'static str> {
    let (key, value) = pair.split_once('=').ok_or("expected KEY=VALUE format")?;
    if key.is_empty() {
        return Err("environment variable name cannot be empty");
    }
    let bytes = key.as_bytes();
    if !(bytes[0].is_ascii_alphabetic() || bytes[0] == b'_') {
        return Err("environment variable name must start with a letter or underscore");
    }
    if !key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err("environment variable name must contain only [A-Za-z0-9_]");
    }
    if value.is_empty() {
        return Err("environment variable value cannot be empty");
    }
    Ok((key, value))
}

/// Default skills directory relative to manifest.
pub const SKILLS_DIR: &str = "skills";

/// Canonical skill entry point filename.
pub const SKILL_FILE_NAME: &str = "SKILL.md";

/// Placeholder skill description used in scaffold template.
pub const DEFAULT_SKILL_DESCRIPTION: &str = "Add your skill description here";

pub fn is_placeholder_skill_description(description: &str) -> bool {
    description == DEFAULT_SKILL_DESCRIPTION
}

/// Skill `SKILL.md` scaffold template.
pub fn skill_template(name: &str, description: &str) -> String {
    format!(
        "\
---
name: {name}
description: \"{description}\"
---

# {name}

## when to use

describe when this skill should be activated

## Instructions

step-by-step guidance for the agent

## Examples

concrete usage examples
"
    )
}

pub fn is_skill_template(content: &str) -> bool {
    content.contains("describe when this skill should be activated")
        && content.contains("step-by-step guidance for the agent")
        && content.contains("concrete usage examples")
}

/// Canonical system prompt filename inside `.theta/`.
pub const SYSTEM_FILE_NAME: &str = "system.md";

/// Typed handle to the `.theta/` materialization directory.
///
/// Declares the expected directory structure as named slots.
/// Pure path resolution — no filesystem IO.
pub struct ThetaProjectLayout {
    root: PathBuf,
}

impl ThetaProjectLayout {
    /// Create a layout rooted at `project_dir/.theta/`.
    pub fn new(project_dir: &Path) -> Self {
        Self {
            root: project_dir.join(DOT_THETA_DIR),
        }
    }

    /// The `.theta/` directory itself.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `.theta/system.md`
    pub fn system(&self) -> PathBuf {
        self.root.join(SYSTEM_FILE_NAME)
    }

    /// `.theta/rules/`
    pub fn rules_dir(&self) -> PathBuf {
        self.root.join("rules")
    }

    /// `.theta/rules/<name>.md`
    pub fn rule(&self, name: &str) -> PathBuf {
        self.root.join("rules").join(format!("{name}.md"))
    }

    /// `.theta/skills/`
    pub fn skills_dir(&self) -> PathBuf {
        self.root.join("skills")
    }

    /// `.theta/skills/<name>/`
    pub fn skill(&self, name: &str) -> PathBuf {
        self.root.join("skills").join(name)
    }

    /// `.theta/subagents/`
    pub fn subagents_dir(&self) -> PathBuf {
        self.root.join(SUBAGENTS_DIR_NAME)
    }

    /// `.theta/subagents/<name>/`
    pub fn subagent(&self, name: &str) -> PathBuf {
        self.root.join(SUBAGENTS_DIR_NAME).join(name)
    }

    /// `.theta/subagents/<name>/theta.toml`
    pub fn subagent_manifest(&self, name: &str) -> PathBuf {
        self.subagent(name).join(MANIFEST_FILE_NAME)
    }

    /// Relative path for a rule: `"rules/safety.md"`.
    pub fn rule_rel(name: &str) -> String {
        format!("rules/{name}.md")
    }

    /// Relative path for a skill dir: `"skills/osint"`.
    pub fn skill_rel(name: &str) -> String {
        format!("skills/{name}")
    }

    /// Relative path for a subagent dir: `"subagents/researcher"`.
    pub fn subagent_rel(name: &str) -> String {
        format!("{SUBAGENTS_DIR_NAME}/{name}")
    }

    /// Relative path for a subagent prompt file: `"subagents/researcher.md"`.
    pub fn subagent_prompt_rel(name: &str) -> String {
        format!("{SUBAGENTS_DIR_NAME}/{name}.md")
    }
}

/// `.theta/subagents/<name>/system.md` — materialized system prompt for both
/// ref and inline subagents.
pub fn subagent_system_rel(name: &str) -> String {
    format!("{SUBAGENTS_DIR_NAME}/{name}/{SYSTEM_FILE_NAME}")
}

/// Canonical section ordering for `theta.toml` — used by import to produce
/// well-ordered output files.
pub const MANIFEST_SECTION_ORDER: &[&str] = &[
    "theta",
    "agent",
    "instructions",
    "skills",
    "tools",
    "subagents",
    "harness",
    "extras",
];

/// Whether `key` is a known top-level `theta.toml` section.
/// Callers that build sections from importers should validate against this
/// to catch typos at the boundary.
pub fn is_known_section(key: &str) -> bool {
    MANIFEST_SECTION_ORDER.contains(&key)
}

/// Strip `base` from `path` and return the relative remainder as a `String`.
///
/// Returns the path as-is (lossy) if it's not under `base`.
/// Used when inserting relative paths into TOML values.
pub fn rel_string(path: &Path, base: &Path) -> String {
    let rel = path.strip_prefix(base).unwrap_or(path);
    rel.to_str()
        .expect("non-UTF-8 path in rel_string - theta requires UTF-8 file paths")
        .to_string()
}

/// Typed handle to the system resource store at `$XDG_DATA_HOME/theta/store/`.
///
/// Pure path resolution, no filesystem IO.
/// Create via `SystemStoreLayout::new(data_dir)` where `data_dir` is from `theta_dirs::data_dir()`.
pub struct SystemStoreLayout {
    root: PathBuf,
}

impl SystemStoreLayout {
    /// Create from the theta data dir: `data_dir/store/`.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            root: data_dir.join("store"),
        }
    }

    /// `store/`
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `store/index.toml`
    pub fn index(&self) -> PathBuf {
        self.root.join("index.toml")
    }

    /// `store/skills/`
    pub fn skills_dir(&self) -> PathBuf {
        self.root.join("skills")
    }

    /// `store/skills/<name>/`
    pub fn skill(&self, name: &str) -> PathBuf {
        self.root.join("skills").join(name)
    }

    /// `store/skills/<name>/SKILL.md`
    pub fn skill_md(&self, name: &str) -> PathBuf {
        self.skill(name).join(SKILL_FILE_NAME)
    }

    /// `store/rules/`
    pub fn rules_dir(&self) -> PathBuf {
        self.root.join("rules")
    }

    /// `store/rules/<name>.md`
    pub fn rule(&self, name: &str) -> PathBuf {
        self.root.join("rules").join(format!("{name}.md"))
    }

    /// `store/agents/`
    pub fn agents_dir(&self) -> PathBuf {
        self.root.join("agents")
    }

    /// `store/agents/<name>/`
    pub fn agent(&self, name: &str) -> PathBuf {
        self.root.join("agents").join(name)
    }

    /// `store/agents/<name>/theta.toml`
    pub fn agent_manifest(&self, name: &str) -> PathBuf {
        self.agent(name).join(MANIFEST_FILE_NAME)
    }

    /// `store/agents/<name>/theta.lock`
    pub fn agent_lock(&self, name: &str) -> PathBuf {
        self.agent(name).join(LOCKFILE)
    }
}

/// Kind of resource in the system store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreResourceKind {
    Skill,
    Rule,
    Agent,
}

impl std::fmt::Display for StoreResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Skill => "skill",
            Self::Rule => "rule",
            Self::Agent => "agent",
        })
    }
}

impl std::str::FromStr for StoreResourceKind {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "skill" => Ok(Self::Skill),
            "rule" => Ok(Self::Rule),
            "agent" => Ok(Self::Agent),
            other => Err(format!(
                "unknown resource type \"{other}\": expected skill, rule, or agent"
            )),
        }
    }
}

/// Entry in the store index for a single registered resource.
#[derive(Debug, Clone)]
pub struct StoreEntry {
    /// ISO 8601 timestamp of initial registration
    pub registered: String,
    /// Absolute path to the project the resource was registered from
    pub source_project: String,
    /// Human-readable description of the resource
    pub description: String,
}

/// The full store index (`store/index.toml`) — provides discovery metadata
/// for all registered skills, rules, and agents.
///
/// This struct is a pure deserialization target, no IO.
#[derive(Debug, Default, Clone)]
pub struct StoreIndex {
    pub skills: std::collections::BTreeMap<String, StoreEntry>,
    pub rules: std::collections::BTreeMap<String, StoreIndexRuleEntry>,
    pub agents: std::collections::BTreeMap<String, StoreEntry>,
}

/// Index entry for a rule (includes apply mode in addition to base fields).
#[derive(Debug, Clone)]
pub struct StoreIndexRuleEntry {
    pub registered: String,
    pub source_project: String,
    pub description: String,
    /// Activation mode stored at registration time (e.g. "always", "model-decision")
    pub apply: Option<String>,
}

/// Recursively copy a directory tree from `src` to `dest`.
/// If `dest` already exists it is removed first (wipe-then-copy semantics).
pub fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        fs_err::remove_dir_all(dest)
            .with_context(|| format!("failed to remove existing {}", dest.display()))?;
    }
    fs_err::create_dir_all(dest).with_context(|| format!("failed to create {}", dest.display()))?;

    for entry in
        fs_err::read_dir(src).with_context(|| format!("failed to read {}", src.display()))?
    {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs_err::copy(&src_path, &dest_path)
                .with_context(|| format!("failed to copy {}", src_path.display()))?;
        }
    }
    Ok(())
}
/// Convert a string to kebab-case via `heck`.
pub fn kebab_case(s: &str) -> String {
    s.to_kebab_case()
}
// Builtin skills — shipped inside the binary, seeded into the system store.

/// A single file inside a builtin skill directory.
pub struct BuiltinFile {
    /// Relative path within the skill dir (e.g. `"SKILL.md"`, `"scripts/setup.sh"`).
    pub path: &'static str,
    /// File content, embedded at compile time via `include_str!`.
    pub content: &'static str,
}

/// A builtin skill: name + description + one or more files.
pub struct BuiltinSkill {
    /// Kebab-case name (also the directory name in the store).
    pub name: &'static str,
    /// Short description for the store index entry.
    pub description: &'static str,
    /// Files to write into `store/skills/<name>/`. Must include `SKILL.md`.
    pub files: &'static [BuiltinFile],
}

/// All builtin skills land seed `theta-store` on installation.
pub const BUILTIN_SKILLS: &[BuiltinSkill] = &[BuiltinSkill {
    name: "use-theta",
    description: "Use the theta CLI to manage agent configurations: add/remove rules, tools, skills, subagents; cast to and from harnesses; validate; lock and materialize dependencies.",
    files: &[BuiltinFile {
        path: "SKILL.md",
        content: include_str!("builtins/use-theta/SKILL.md"),
    }],
}];

/// Split `---\nyaml\n---\nbody` into (`yaml_str`, `body`).
/// Returns `(None, full_input)` if no frontmatter delimiters are found.
pub fn split_frontmatter(input: &str) -> (Option<&str>, &str) {
    let stripped = input.trim_start_matches('\u{feff}').trim_start();
    let trimmed = stripped.strip_prefix("---").unwrap_or(stripped);
    if std::ptr::eq(trimmed, stripped) {
        return (None, input);
    }
    let Some(end) = trimmed.find("\n---") else {
        return (None, input);
    };
    let yaml = trimmed[..end].trim();
    let after = &trimmed[end + 4..];
    let body = after.strip_prefix('\n').unwrap_or(after);
    if yaml.is_empty() {
        (None, body)
    } else {
        (Some(yaml), body)
    }
}
