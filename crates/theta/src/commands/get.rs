//! `theta get` — emit all materialized project content as JSON.
//!
//! Reads the manifest, lockfile, and `.theta/` directory and serialises
//! everything into a single `GetOutcome` envelope. Requires `theta sync`
//! to have been run first (`.theta/` must exist).
//!
//! The output schema is separately emittable via `theta schema --get`, which
//! lets `theta_py._codegen` generate typed Pydantic models for it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::Serialize;
use theta_args::{GetArgs, OutputFormat};
use theta_lock::read_lock;
use theta_manifest::read_manifest;
use theta_schema::{ApplyMode, SkillFrontmatter, ThetaManifest};
use theta_static::{
    DOT_THETA_DIR, LOCKFILE, RULES_DIR, SKILL_FILE_NAME, SKILLS_DIR, SYSTEM_FILE_NAME,
};

use super::output::present;
use super::{project_dir, require_manifest};

/// The full materialized state of a theta project.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct GetOutcome {
    /// Agent identity fields from `theta.toml [agent]`.
    pub agent: AgentInfo,
    /// Canonical hash of the `theta.toml` that produced the current lock.
    /// `null` when `theta.lock` does not exist (project not yet locked).
    pub lock_hash: Option<String>,
    /// Rendered system prompt from `.theta/system.md`.
    /// `null` when no system prompt is configured.
    pub system_prompt: Option<String>,
    /// Materialized rules, keyed by rule name.
    pub rules: BTreeMap<String, MaterializedRule>,
    /// Materialized skills, keyed by skill name.
    pub skills: BTreeMap<String, MaterializedSkill>,
    /// MCP tool configs from `theta.toml [tools]`.
    pub tools: BTreeMap<String, MaterializedTool>,
}

/// Agent identity from `theta.toml [agent]`.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct AgentInfo {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub model: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// A single materialized rule — content plus the metadata from `theta.toml`.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct MaterializedRule {
    /// Full text content of the rule.
    pub content: String,
    /// Application mode declared in `theta.toml`.
    pub apply: String,
    /// Glob patterns for `apply: glob` rules.
    pub apply_to: Option<Vec<String>>,
    /// Human-readable summary from `theta.toml`.
    pub summary: Option<String>,
    /// Description from `theta.toml` (used for `apply: model-decision`).
    pub description: Option<String>,
}

/// A single materialized skill — body plus frontmatter, plus the `tags` and
/// `goal` declared in `theta.toml`, plus all non-SKILL.md supporting files.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct MaterializedSkill {
    /// `name` from SKILL.md frontmatter.
    pub name: String,
    /// `description` from SKILL.md frontmatter.
    pub description: Option<String>,
    /// Full text of SKILL.md (including frontmatter).
    pub skill_md: String,
    /// `tags` from `theta.toml [skills.<name>]`.
    pub tags: Option<Vec<String>>,
    /// `goal` from `theta.toml [skills.<name>]`.
    pub goal: Option<String>,
    /// All other files in the skill directory (relative path → content).
    /// Only UTF-8 readable files are included; binary files are skipped.
    pub supporting_files: BTreeMap<String, String>,
    /// Path to the skill directory inside `.theta/skills/`.
    /// Pass directly to `harbor run --skill`.
    pub path: PathBuf,
}

/// A single MCP tool config from `theta.toml [tools]`.
#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct MaterializedTool {
    pub command: Option<Vec<String>>,
    pub url: Option<String>,
    pub env: Option<BTreeMap<String, String>>,
    pub args: Option<Vec<String>>,
    pub enabled: bool,
}

pub(crate) fn execute(
    _args: GetArgs,
    output_format: OutputFormat,
    manifest_path: &Path,
) -> Result<()> {
    require_manifest(manifest_path)?;

    let project_dir = project_dir(manifest_path)?;
    let out_dir = std::env::var(theta_static::THETA_OUT_DIR_ENV)
        .ok()
        .map_or_else(|| project_dir.to_path_buf(), std::path::PathBuf::from);
    let theta_dir = out_dir.join(DOT_THETA_DIR);

    if !theta_dir.exists() {
        anyhow::bail!(
            ".theta/ not found in {} — run `theta sync` first",
            out_dir.display()
        );
    }

    let manifest = read_manifest(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;

    let lock_hash = read_lock_hash(&out_dir);
    let agent = build_agent_info(&manifest);
    let system_prompt = read_system_prompt(&theta_dir);
    let rules = build_rules(&manifest, &theta_dir);
    let skills = build_skills(&manifest, &theta_dir);
    let tools = build_tools(&manifest);

    let outcome = GetOutcome {
        agent,
        lock_hash,
        system_prompt,
        rules,
        skills,
        tools,
    };

    present(&["get"], output_format, outcome, vec![], |_| {
        // human output: nothing useful to print; content is in JSON
    })
}

fn read_lock_hash(project_dir: &Path) -> Option<String> {
    let lock_path = project_dir.join(LOCKFILE);
    let lock = read_lock(&lock_path).ok()?;
    Some(lock.meta.manifest_hash.to_string())
}

fn build_agent_info(manifest: &ThetaManifest) -> AgentInfo {
    AgentInfo {
        name: manifest.agent.name.clone(),
        description: manifest.agent.description.clone(),
        version: manifest.agent.version.clone(),
        authors: manifest.agent.authors.clone(),
        model: manifest.agent.model.clone(),
        tags: manifest.agent.tags.clone(),
    }
}

fn read_system_prompt(theta_dir: &Path) -> Option<String> {
    let path = theta_dir.join(SYSTEM_FILE_NAME);
    fs_err::read_to_string(&path).ok()
}

fn build_rules(manifest: &ThetaManifest, theta_dir: &Path) -> BTreeMap<String, MaterializedRule> {
    let rules_dir = theta_dir.join(RULES_DIR);
    let manifest_rules = manifest
        .instructions
        .as_ref()
        .and_then(|i| i.rules.as_ref());

    let mut result = BTreeMap::new();

    // Walk `.theta/rules/`,  only include rules that are actually materialized.
    if !rules_dir.exists() {
        return result;
    }

    for entry in walkdir_md(&rules_dir) {
        let rel = entry.strip_prefix(&rules_dir).unwrap_or(&entry);
        let name = rel.with_extension("").to_string_lossy().replace('\\', "/");

        let Ok(content) = fs_err::read_to_string(&entry) else {
            continue;
        };

        let (apply, apply_to, summary, description) = if let Some(rules) = manifest_rules {
            if let Some(rule) = rules.get(&name) {
                (
                    rule.apply.as_str().to_string(),
                    rule.apply_to.clone(),
                    rule.summary.clone(),
                    rule.description.clone(),
                )
            } else {
                (ApplyMode::default().as_str().to_string(), None, None, None)
            }
        } else {
            (ApplyMode::default().as_str().to_string(), None, None, None)
        };

        result.insert(
            name,
            MaterializedRule {
                content,
                apply,
                apply_to,
                summary,
                description,
            },
        );
    }

    result
}

fn build_skills(manifest: &ThetaManifest, theta_dir: &Path) -> BTreeMap<String, MaterializedSkill> {
    let skills_dir = theta_dir.join(SKILLS_DIR);
    if !skills_dir.exists() {
        return BTreeMap::new();
    }

    let manifest_skills = manifest.skills.as_ref();
    let mut result = BTreeMap::new();

    let Ok(entries) = fs_err::read_dir(&skills_dir) else {
        return result;
    };

    for entry in entries.flatten() {
        let skill_dir = entry.path();
        if !skill_dir.is_dir() {
            continue;
        }
        let name = skill_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if name.is_empty() {
            continue;
        }

        let skill_md_path = skill_dir.join(SKILL_FILE_NAME);
        let Ok(skill_md) = fs_err::read_to_string(&skill_md_path) else {
            continue;
        };

        let (fm_name, fm_description) = parse_skill_frontmatter(&skill_md, &name);

        let (tags, goal) = manifest_skills
            .and_then(|skills| skills.get(&name))
            .map_or((None, None), |s| (s.tags.clone(), s.goal.clone()));

        let supporting_files = collect_supporting_files(&skill_dir, &skill_md_path);

        result.insert(
            name,
            MaterializedSkill {
                name: fm_name,
                description: fm_description,
                skill_md,
                tags,
                goal,
                supporting_files,
                path: skill_dir,
            },
        );
    }

    result
}

fn build_tools(manifest: &ThetaManifest) -> BTreeMap<String, MaterializedTool> {
    let Some(ref tools) = manifest.tools else {
        return BTreeMap::new();
    };
    tools
        .iter()
        .map(|(name, tool)| {
            (
                name.clone(),
                MaterializedTool {
                    command: tool.command.clone(),
                    url: tool.url.clone(),
                    env: tool.env.clone(),
                    args: tool.args.clone(),
                    enabled: tool.enabled,
                },
            )
        })
        .collect()
}

/// Parse name and description from SKILL.md frontmatter.
/// Falls back to the directory name if frontmatter is absent or malformed.
fn parse_skill_frontmatter(skill_md: &str, fallback_name: &str) -> (String, Option<String>) {
    let Some(yaml_str) = theta_static::split_frontmatter(skill_md).0 else {
        return (fallback_name.to_string(), None);
    };
    let fm = SkillFrontmatter::parse(yaml_str).unwrap_or(SkillFrontmatter {
        name: None,
        description: None,
    });
    (
        fm.name.unwrap_or_else(|| fallback_name.to_string()),
        fm.description.filter(|d| !d.is_empty()),
    )
}

/// Collect all UTF-8 readable files in `skill_dir` except `SKILL.md`,
/// keyed by their path relative to `skill_dir`.
fn collect_supporting_files(skill_dir: &Path, skip: &Path) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    collect_supporting_files_recursive(skill_dir, skill_dir, skip, &mut result);
    result
}

fn collect_supporting_files_recursive(
    root: &Path,
    dir: &Path,
    skip: &Path,
    out: &mut BTreeMap<String, String>,
) {
    let Ok(entries) = fs_err::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path == skip {
            continue;
        }
        if path.is_dir() {
            collect_supporting_files_recursive(root, &path, skip, out);
        } else if let Ok(content) = fs_err::read_to_string(&path) {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(rel, content);
        }
    }
}

/// Walk a directory and return all `.md` file paths (recursive).
fn walkdir_md(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    walkdir_md_recursive(dir, &mut result);
    result.sort();
    result
}

fn walkdir_md_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs_err::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walkdir_md_recursive(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
}
