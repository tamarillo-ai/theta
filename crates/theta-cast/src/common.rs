use std::path::{Path, PathBuf};
use theta_static::kebab_case;

use anyhow::{Context, Result};
use theta_schema::{Diagnostic, ThetaManifest};
use tracing::{debug, instrument, warn};

pub type CastFile = (PathBuf, CastContent);

pub(crate) use theta_static::split_frontmatter;

#[derive(Debug, Clone)]
pub enum CastContent {
    Text(String),
    Binary(Vec<u8>),
}

impl std::ops::Deref for CastContent {
    type Target = str;

    fn deref(&self) -> &str {
        match self {
            Self::Text(s) => s,
            Self::Binary(b) => std::str::from_utf8(b).unwrap_or("<binary>"),
        }
    }
}

impl AsRef<[u8]> for CastContent {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Text(s) => s.as_bytes(),
            Self::Binary(b) => b,
        }
    }
}

impl From<String> for CastContent {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<Vec<u8>> for CastContent {
    fn from(b: Vec<u8>) -> Self {
        Self::Binary(b)
    }
}

/// Parse JSONC — JSON with comments and trailing commas.
///
/// Some harnesses treat their `*.json` config as JSONC: VS Code's
/// `settings.json`/`mcp.json`, Claude Code's `.claude/settings.json`, and others
/// allow comments and trailing commas.
/// Strict `serde_json::from_str` rejects these and blows up.
pub(crate) fn parse_jsonc_value(raw: &str, path: &Path) -> Result<serde_json::Value> {
    jsonc_parser::parse_to_serde_value(raw, &jsonc_parser::ParseOptions::default())
        .with_context(|| format!("failed to parse {}", path.display()))
}

pub(crate) fn parse_jsonc_map(
    raw: &str,
    path: &Path,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    let value: serde_json::Value =
        jsonc_parser::parse_to_serde_value(raw, &jsonc_parser::ParseOptions::default())
            .with_context(|| format!("failed to parse {}", path.display()))?;
    match value {
        serde_json::Value::Object(map) => Ok(map),
        serde_json::Value::Null => Ok(serde_json::Map::new()),
        other => {
            let kind = match other {
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                _ => "non-object",
            };
            anyhow::bail!(
                "{} is valid JSON but not an object (found {kind})",
                path.display()
            )
        }
    }
}

pub(crate) fn read_existing_json_map(
    path: &Path,
) -> Result<Option<serde_json::Map<String, serde_json::Value>>> {
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs_err::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let map = parse_jsonc_map(&raw, path)?;
    Ok(Some(map))
}

fn read_theta_file(theta_dir: &Path, rel_path: &str) -> Result<String> {
    let full = theta_dir.join(rel_path);
    fs_err::read_to_string(&full).with_context(|| format!("failed to read {}", full.display()))
}

#[instrument(skip_all, fields(theta_dir = %theta_dir.display()))]
pub(crate) fn build_system_prompt(
    manifest: &ThetaManifest,
    theta_dir: &Path,
) -> Result<Option<String>> {
    let has_system = manifest
        .instructions
        .as_ref()
        .is_some_and(|i| i.system.is_some());
    if !has_system {
        return Ok(None);
    }
    let content = read_theta_file(theta_dir, theta_static::SYSTEM_FILE_NAME)?;
    Ok(Some(content))
}

#[instrument(skip_all, fields(theta_dir = %theta_dir.display()))]
pub(crate) fn identity_and_system(
    manifest: &ThetaManifest,
    theta_dir: &Path,
) -> Result<Vec<String>> {
    let mut sections = Vec::new();
    sections.push(format!(
        "# {}\n\n{}",
        manifest.agent.name, manifest.agent.description
    ));
    if let Some(content) = build_system_prompt(manifest, theta_dir)? {
        sections.push(content);
    }
    Ok(sections)
}

// TODO: ApplyMode --> frontmatter dispatch is repeated in copilot/cursor/claude_code casters,
// each mapping Always/Glob/ModelDecision/Manual to different harness-native keys.
// Candidate improvement: a declarative macro (or trait method on a HarnessLayout) that takes
// an ApplyMode + apply_to patterns and returns Vec<(&str, Value)> frontmatter entries,
// so each harness declares a key table instead of an imperative match block.
#[instrument(skip_all, fields(theta_dir = %theta_dir.display()))]
pub(crate) fn read_all_rules(
    manifest: &ThetaManifest,
    theta_dir: &Path,
) -> Result<Vec<(String, theta_schema::Rule, String)>> {
    let mut out = Vec::new();
    if let Some(ref instructions) = manifest.instructions {
        if let Some(ref rules) = instructions.rules {
            for (name, rule) in rules {
                let rel = theta_static::ThetaProjectLayout::rule_rel(name);
                let content = read_theta_file(theta_dir, &rel)
                    .with_context(|| format!("failed to load rule \"{name}\""))?;
                out.push((name.clone(), rule.clone(), content));
            }
        }
    }
    Ok(out)
}

pub(crate) fn fm_str(s: impl Into<String>) -> serde_norway::Value {
    serde_norway::Value::String(s.into())
}

pub(crate) fn fm_bool(b: bool) -> serde_norway::Value {
    serde_norway::Value::Bool(b)
}

pub(crate) fn fm_list(items: &[String]) -> serde_norway::Value {
    serde_norway::Value::Sequence(
        items
            .iter()
            .map(|s| serde_norway::Value::String(s.clone()))
            .collect(),
    )
}

pub(crate) fn yaml_frontmatter(entries: &[(&str, serde_norway::Value)]) -> Result<String> {
    if entries.is_empty() {
        return Ok(String::new());
    }
    let map: serde_norway::Mapping = entries
        .iter()
        .map(|(k, v)| (serde_norway::Value::String(k.to_string()), v.clone()))
        .collect();
    let yaml = serde_norway::to_string(&map).context("frontmatter serialization failed")?;
    Ok(format!("---\n{yaml}---\n"))
}

pub(crate) struct ParsedFrontmatter {
    pub data: serde_json::Map<String, serde_json::Value>,
    pub content: String,
}

pub(crate) fn parse_frontmatter(input: &str) -> ParsedFrontmatter {
    let (yaml_str, body) = split_frontmatter(input);
    let Some(yaml_str) = yaml_str else {
        return ParsedFrontmatter {
            data: serde_json::Map::new(),
            content: body.to_string(),
        };
    };
    match serde_norway::from_str::<serde_json::Value>(yaml_str) {
        Ok(serde_json::Value::Object(map)) => ParsedFrontmatter {
            data: map,
            content: body.to_string(),
        },
        Ok(_) => {
            warn!("frontmatter is valid YAML but not a mapping, treating as plain content");
            ParsedFrontmatter {
                data: serde_json::Map::new(),
                content: body.to_string(),
            }
        }
        Err(e) => {
            warn!(error = %e, "frontmatter YAML parse failed - fields not imported");
            ParsedFrontmatter {
                data: serde_json::Map::new(),
                content: body.to_string(),
            }
        }
    }
}

impl ParsedFrontmatter {
    pub(crate) fn get_str(&self, key: &str) -> Option<&str> {
        self.data.get(key).and_then(|v| v.as_str())
    }

    pub(crate) fn get_bool(&self, key: &str) -> Option<bool> {
        self.data.get(key).and_then(serde_json::Value::as_bool)
    }

    pub(crate) fn get_str_list(&self, key: &str) -> Option<Vec<String>> {
        let val = self.data.get(key)?;
        match val {
            serde_json::Value::Array(arr) => Some(
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
            ),
            // YAML string --> single-element list. harness-specific splitting
            // (e.g. cursor's comma-separated globs) is handled by the
            // harness's own parser, not here.
            serde_json::Value::String(s) => Some(vec![s.clone()]),
            _ => None,
        }
    }
}

// shape of the identity header at the top of a system prompt file
// used to emit diagnostics when the caller wants to warn about ambiguous
// parses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdentityHeaderShape {
    // no `# Heading` at the top of the file.
    NoHeading,
    // `# Heading` + blank line + description + blank line + body
    WellFormed,
    // `# Heading` only, nothing else (or only whitespace) after it.
    HeadingOnly,
    // `# Heading` followed by text but no blank line separator.
    // the "description" consumed everything after the heading and
    // the body is empty. this is almost always a malformed file.
    HeadingNoBlankLine,
}

pub(crate) fn strip_identity_header_with_shape(
    content: &str,
) -> (Option<String>, Option<String>, String, IdentityHeaderShape) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("# ") {
        return (
            None,
            None,
            content.to_string(),
            IdentityHeaderShape::NoHeading,
        );
    }
    let heading_end = trimmed.find('\n').unwrap_or(trimmed.len());
    let name = trimmed[2..heading_end].trim().to_string();

    let after_heading = trimmed[heading_end..].trim_start_matches('\n');
    if after_heading.trim().is_empty() {
        return (
            Some(name),
            None,
            String::new(),
            IdentityHeaderShape::HeadingOnly,
        );
    }
    if let Some(blank) = after_heading.find("\n\n") {
        let desc = after_heading[..blank].trim().to_string();
        let body = after_heading[blank..].trim_start_matches('\n').to_string();
        let desc_opt = if desc.is_empty() { None } else { Some(desc) };
        (Some(name), desc_opt, body, IdentityHeaderShape::WellFormed)
    } else {
        let desc = after_heading.trim().to_string();
        let desc_opt = if desc.is_empty() { None } else { Some(desc) };
        (
            Some(name),
            desc_opt,
            String::new(),
            IdentityHeaderShape::HeadingNoBlankLine,
        )
    }
}

pub(crate) fn default_agent_name(project_dir: &Path) -> String {
    let dirname = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unnamed");
    kebab_case(&format!("{dirname}-agent"))
}

pub(crate) fn new_import_document() -> toml_edit::DocumentMut {
    let mut doc = toml_edit::DocumentMut::new();

    let mut theta = toml_edit::Table::new();
    theta["schema"] = toml_edit::value(theta_static::SCHEMA_VERSION);
    doc["theta"] = toml_edit::Item::Table(theta);

    doc
}

pub(crate) fn set_import_agent(doc: &mut toml_edit::DocumentMut, name: &str, description: &str) {
    let mut agent = toml_edit::Table::new();
    agent["name"] = toml_edit::value(kebab_case(name));
    agent["description"] = toml_edit::value(description);
    doc["agent"] = toml_edit::Item::Table(agent);
}

#[instrument(skip_all, fields(theta_dir = %theta_dir.display(), name))]
pub(crate) fn read_skill_dir_files(
    theta_dir: &Path,
    name: &str,
    harness_skills_dir: &Path,
) -> Result<Vec<CastFile>> {
    let skill_src = theta_dir.join(theta_static::SKILLS_DIR).join(name);
    if !skill_src.is_dir() {
        anyhow::bail!("skill \"{name}\" not found at {}", skill_src.display());
    }
    let mut files = Vec::new();
    collect_dir_files(
        &skill_src,
        &skill_src,
        &harness_skills_dir.join(name),
        &mut files,
    )?;
    Ok(files)
}

fn collect_dir_files(
    root: &Path,
    dir: &Path,
    target_prefix: &Path,
    out: &mut Vec<CastFile>,
) -> Result<()> {
    let mut entries: Vec<_> = fs_err::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dir_files(root, &path, target_prefix, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .with_context(|| format!("{} is not under {}", path.display(), root.display()))?;
            let rel_path = target_prefix.join(rel);
            if let Ok(content) = fs_err::read_to_string(&path) {
                out.push((rel_path, CastContent::Text(content)));
            } else {
                // majority of skills assets will be non binary files
                // ref: https://cursor.com/docs/skills#optional-directories
                let bytes = fs_err::read(&path)
                    .with_context(|| format!("failed to read {}", path.display()))?;
                out.push((rel_path, CastContent::Binary(bytes)));
            }
        }
    }
    Ok(())
}

#[instrument(skip_all, fields(project_dir = %project_dir.display(), skills_dir = %skills_dir.display()))]
pub(crate) fn import_skills_from_dir(
    project_dir: &Path,
    skills_dir: &Path,
) -> Result<Vec<(String, Vec<CastFile>, PathBuf)>> {
    let abs_skills = project_dir.join(skills_dir);
    if !abs_skills.is_dir() {
        debug!(path = %abs_skills.display(), "skills dir not found, skipping");
        return Ok(Vec::new());
    }

    let mut results = Vec::new();
    let mut entries: Vec<_> = fs_err::read_dir(&abs_skills)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(fs_err::DirEntry::file_name);

    for entry in entries {
        let skill_name = filename_to_string(&entry)?;
        let skill_md = entry.path().join(theta_static::SKILL_FILE_NAME);
        if !skill_md.exists() {
            continue;
        }
        let skill_dir = entry.path();
        let mut cast_files = Vec::new();
        collect_dir_files(
            &skill_dir,
            &skill_dir,
            &PathBuf::from(theta_static::ThetaProjectLayout::skill_rel(&skill_name)),
            &mut cast_files,
        )?;
        let source_rel = skills_dir.join(&skill_name);
        results.push((skill_name, cast_files, source_rel));
    }

    Ok(results)
}

#[cfg(test)]
pub(crate) fn json_to_toml_string(value: &serde_json::Value) -> Result<String> {
    let toml_val: toml::Value =
        serde_json::from_value(value.clone()).context("json value is not representable as TOML")?;
    toml::to_string(&toml_val).context("failed to serialize as TOML")
}

/// Convert a JSON value to a `toml_edit::Item`, stripping nulls.
///
/// JSON --> TOML conversion limitations:
///
/// - `null` values are dropped (TOML has no null type)
/// - Numbers outside `i64`/`f64` range are dropped
/// - Objects become inline tables: `{a = 1, b = "two"}`
/// - Arrays become TOML arrays (may contain mixed types, which is
///   valid in TOML v1.1 but not v1.0 — `toml_edit` accepts both)
pub(crate) fn json_to_toml_item(value: &serde_json::Value) -> toml_edit::Item {
    let cleaned = strip_json_nulls(value);
    json_to_toml_item_inner(&cleaned)
}

pub(crate) fn json_to_toml_item_with_diagnostics(
    value: &serde_json::Value,
    context: &str,
    diags: &mut Vec<Diagnostic>,
) -> toml_edit::Item {
    if value.is_null() {
        diags.push(Diagnostic::hint(
            context.to_string(),
            "null value omitted (TOML v1.1 has no null type - key omitted)",
        ));
        return toml_edit::Item::None;
    }
    let (cleaned, null_paths) = strip_json_nulls_tracked(value, context);
    for path in &null_paths {
        diags.push(Diagnostic::hint(
            path.clone(),
            "null value omitted (TOML v1.1 has no null type - key omitted)",
        ));
    }
    let item = json_to_toml_item_inner(&cleaned);
    if item.is_none() {
        diags.push(Diagnostic::hint(
            context.to_string(),
            "value dropped (not representable in TOML)",
        ));
    }
    item
}

/// Recursive JSON --> `toml_edit::Value` conversion.
/// Returns `None` for null and unconvertible numbers.
fn json_to_toml_value(value: &serde_json::Value) -> Option<toml_edit::Value> {
    match value {
        serde_json::Value::String(s) => Some(toml_edit::Value::from(s.as_str())),
        serde_json::Value::Bool(b) => Some(toml_edit::Value::from(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(toml_edit::Value::from(i))
            } else {
                n.as_f64().map(toml_edit::Value::from)
            }
        }
        serde_json::Value::Null => None,
        serde_json::Value::Array(arr) => {
            let mut toml_arr = toml_edit::Array::new();
            for item in arr {
                if let Some(v) = json_to_toml_value(item) {
                    toml_arr.push(v);
                }
            }
            Some(toml_edit::Value::Array(toml_arr))
        }
        serde_json::Value::Object(map) => {
            let mut table = toml_edit::InlineTable::new();
            for (k, v) in map {
                if let Some(val) = json_to_toml_value(v) {
                    table.insert(k, val);
                }
            }
            Some(toml_edit::Value::InlineTable(table))
        }
    }
}

fn json_to_toml_item_inner(value: &serde_json::Value) -> toml_edit::Item {
    if let serde_json::Value::Object(map) = value {
        let mut table = toml_edit::Table::new();
        for (k, v) in map {
            let item = json_to_toml_item_inner(v);
            if !item.is_none() {
                table.insert(k, item);
            }
        }
        return toml_edit::Item::Table(table);
    }
    match json_to_toml_value(value) {
        Some(v) => toml_edit::Item::Value(v),
        None => toml_edit::Item::None,
    }
}

pub(crate) fn strip_json_nulls(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let cleaned: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k.clone(), strip_json_nulls(v)))
                .collect();
            serde_json::Value::Object(cleaned)
        }
        serde_json::Value::Array(arr) => {
            let cleaned: Vec<serde_json::Value> = arr
                .iter()
                .filter(|v| !v.is_null())
                .map(strip_json_nulls)
                .collect();
            serde_json::Value::Array(cleaned)
        }
        other => other.clone(),
    }
}

fn strip_json_nulls_tracked(
    value: &serde_json::Value,
    prefix: &str,
) -> (serde_json::Value, Vec<String>) {
    let mut removed = Vec::new();
    let cleaned = strip_nulls_inner(value, prefix, &mut removed);
    (cleaned, removed)
}

fn strip_nulls_inner(
    value: &serde_json::Value,
    prefix: &str,
    removed: &mut Vec<String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let cleaned: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter_map(|(k, v)| {
                    let path = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    if v.is_null() {
                        removed.push(path);
                        None
                    } else {
                        Some((k.clone(), strip_nulls_inner(v, &path, removed)))
                    }
                })
                .collect();
            serde_json::Value::Object(cleaned)
        }
        serde_json::Value::Array(arr) => {
            let cleaned: Vec<serde_json::Value> = arr
                .iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    let path = format!("{prefix}[{i}]");
                    if v.is_null() {
                        removed.push(path);
                        None
                    } else {
                        Some(strip_nulls_inner(v, &path, removed))
                    }
                })
                .collect();
            serde_json::Value::Array(cleaned)
        }
        other => other.clone(),
    }
}

// resolved subagent: uniform interface for ref + inline subagents

/// Internal: child manifest + `theta_dir` for ref subagents.
struct RefData {
    manifest: ThetaManifest,
    theta_dir: PathBuf,
}

/// A resolved subagent — ref or inline — exposing a uniform interface.
///
/// Casters use this instead of reaching into the raw `Subagent` schema and
/// manually loading ref data. The merge logic (ref wins for description/model,
/// inline owns `tools/skills`) lives here.
///
/// Currently only identity (description, model) and body (system prompt) are
/// consumed by harness casters. No harness format supports embedding a
/// subagent's rules or skills inline — future getters can be added here when
/// a harness needs them, using the existing manifest-based helpers
/// (`read_all_rules`, `read_skill_dir_files`, etc.).
pub(crate) struct ResolvedSubagent<'a> {
    inline: &'a theta_schema::Subagent,
    theta_dir: &'a Path,
    ref_data: Option<RefData>,
}

impl<'a> ResolvedSubagent<'a> {
    /// Load a resolved subagent: if `agent_ref` is set, reads the child manifest.
    #[instrument(skip_all, fields(name = %subagent.name))]
    pub(crate) fn load(subagent: &'a theta_schema::Subagent, theta_dir: &'a Path) -> Self {
        let ref_data = subagent.agent_ref.as_ref().and_then(|_| {
            let subagent_dir =
                theta_dir.join(theta_static::SUBAGENTS_DIR_NAME).join(&subagent.name);
            let manifest_path = subagent_dir.join("theta.toml");
            let content = match fs_err::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(e) => {
                    warn!(path = %manifest_path.display(), error = %e, "failed to read subagent manifest");
                    return None;
                }
            };
            let manifest: ThetaManifest = match toml::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    warn!(path = %manifest_path.display(), error = %e, "failed to parse subagent manifest");
                    return None;
                }
            };
            Some(RefData {
                manifest,
                theta_dir: subagent_dir,
            })
        });
        ResolvedSubagent {
            inline: subagent,
            theta_dir,
            ref_data,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.inline.name
    }

    /// Ref description wins if non-empty, otherwise inline.
    /// Returns the empty string when neither source carries a description.
    pub(crate) fn description(&self) -> &str {
        self.ref_data
            .as_ref()
            .map(|r| r.manifest.agent.description.as_str())
            .filter(|d| !d.is_empty())
            .unwrap_or_else(|| self.inline.description.as_deref().unwrap_or(""))
    }

    /// Ref model wins if present, otherwise inline.
    pub(crate) fn model(&self) -> Option<&str> {
        self.ref_data
            .as_ref()
            .and_then(|r| r.manifest.agent.model.as_deref())
            .or(self.inline.model.as_deref())
    }

    /// Resolve the body text system prompt from all sources:
    ///
    /// - Ref subagent's system prompt (via `build_system_prompt` on child manifest)
    /// - Materialized `.theta/subagents/<name>/system.md` (stripped of frontmatter)
    /// - Caller-provided fallback
    pub(crate) fn body(&self, fallback: &str) -> Result<String> {
        if let Some(ref r) = self.ref_data {
            if let Some(content) = build_system_prompt(&r.manifest, &r.theta_dir)? {
                if !content.trim().is_empty() {
                    return Ok(content);
                }
            }
        }
        if let Some(body) = self.load_materialized_system_md() {
            return Ok(body);
        }
        Ok(fallback.to_string())
    }

    pub(crate) fn tools(&self) -> Option<&[String]> {
        self.inline.tools.as_deref()
    }

    pub(crate) fn skills(&self) -> Option<&[String]> {
        self.inline.skills.as_deref()
    }

    /// Load `.theta/subagents/<name>/system.md` and strip frontmatter.
    fn load_materialized_system_md(&self) -> Option<String> {
        let path = self
            .theta_dir
            .join(theta_static::SUBAGENTS_DIR_NAME)
            .join(&self.inline.name)
            .join(theta_static::SYSTEM_FILE_NAME);
        let Ok(raw) = fs_err::read_to_string(&path) else {
            debug!(path = %path.display(), "subagent system.md not found");
            return None;
        };
        let parsed = parse_frontmatter(&raw);
        let body = parsed.content.trim().to_string();
        if body.is_empty() { None } else { Some(body) }
    }
}

/// Write a subagent prompt body to `<dir>/<name>.md`.
///
/// - If the file exists with identical content: no-op
/// - If the file exists with different content and `force` is false: error
/// - Returns the relative path as a `String` (not `PathBuf`) because callers
///   insert it directly into TOML values via `toml_edit::value()`
#[instrument(skip_all, fields(dir = %dir.display(), name))]
pub(crate) fn write_subagent_prompt(
    dir: &Path,
    name: &str,
    body: &str,
    force: bool,
    manifest_dir: &Path,
) -> Result<String> {
    fs_err::create_dir_all(dir)
        .with_context(|| format!("failed to create subagent prompts dir: {}", dir.display()))?;

    let target = dir.join(format!("{name}.md"));

    if target.exists() {
        let existing = fs_err::read_to_string(&target)
            .with_context(|| format!("failed to read {}", target.display()))?;
        if existing != body {
            if force {
                fs_err::write(&target, body)
                    .with_context(|| format!("failed to overwrite {}", target.display()))?;
            } else {
                anyhow::bail!(
                    "{} already exists with different content - pass --force-overwrite to replace",
                    target.display()
                );
            }
        }
    } else {
        fs_err::write(&target, body)
            .with_context(|| format!("failed to write {}", target.display()))?;
    }

    Ok(theta_static::rel_string(&target, manifest_dir))
}

pub(crate) fn toml_str_to_json(toml_str: &str) -> Result<serde_json::Value> {
    // anyhow::Error::from preserves the source error chain (line/column from
    // toml::de::Error), so callers that print with `{:#}` or walk `.chain()`
    // see the real parser diagnostic, not a flattened "failed to parse TOML"
    // message.
    let toml_val: toml::Value = toml::from_str(toml_str)
        .map_err(|e| anyhow::Error::from(e).context("failed to parse TOML"))?;
    serde_json::to_value(toml_val).context("failed to convert TOML to JSON")
}

pub(crate) fn merge_json_objects(
    base: serde_json::Map<String, serde_json::Value>,
    overlay: serde_json::Map<String, serde_json::Value>,
    context: &str,
    diags: &mut Vec<Diagnostic>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut out = base;
    for (k, v) in overlay {
        if let Some(existing) = out.get(&k) {
            // NOTE: deep equality check, acceptable because harness extras are usually small
            if existing != &v {
                let path = if context.is_empty() {
                    k.clone()
                } else {
                    format!("{context} --> {k}")
                };
                diags.push(Diagnostic::warn(
                    path,
                    "conflicting value: theta-typed key overrides harness-extras key",
                ));
            }
        }
        out.insert(k, v);
    }
    out
}

fn section_rank(key: &str) -> usize {
    theta_static::MANIFEST_SECTION_ORDER
        .iter()
        .position(|&k| k == key)
        .unwrap_or(theta_static::MANIFEST_SECTION_ORDER.len())
}

pub(crate) fn reorder_import_document(doc: &mut toml_edit::DocumentMut) {
    let mut entries: Vec<(toml_edit::Key, toml_edit::Item)> = Vec::new();
    let keys: Vec<String> = doc.as_table().iter().map(|(k, _)| k.to_string()).collect();
    for key in &keys {
        if let Some((k, item)) = doc.as_table_mut().remove_entry(key) {
            entries.push((k, item));
        }
    }
    entries.sort_by_key(|(k, _)| section_rank(k.get()));
    for (key, item) in entries {
        doc.as_table_mut().insert_formatted(&key, item);
    }
}
/// Convert an `OsString` file name to a `String`, returning an error if it contains non-UTF-8.
pub(crate) fn filename_to_string(entry: &fs_err::DirEntry) -> Result<String> {
    entry
        .file_name()
        .into_string()
        .map_err(|os| anyhow::anyhow!("non-UTF-8 filename: {os:?}"))
}

/// Extract file stem as a UTF-8 `String` from a path, error on non-UTF-8.
pub(crate) fn file_stem_str(path: &Path) -> Result<String> {
    path.file_stem()
        .ok_or_else(|| anyhow::anyhow!("path has no file stem: {}", path.display()))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("non-UTF-8 file stem: {}", path.display()))
        .map(String::from)
}

/// Append content to the system-prompt extracted file, or create one if it doesn't exist.
/// Also ensures `[instructions].system` is set in the document.
/// Used by cross-read importers to concatenate `AGENTS.md` / `CLAUDE.md` into the system prompt.
pub(crate) fn append_cross_read_to_system_prompt(
    doc: &mut toml_edit::DocumentMut,
    extracted: &mut Vec<CastFile>,
    content: &str,
) {
    let sys_path = PathBuf::from(theta_static::SYSTEM_FILE_NAME);
    if let Some(entry) = extracted.iter_mut().find(|(p, _)| p == &sys_path) {
        let existing: &str = &entry.1;
        let combined = format!("{}\n\n{}\n", existing.trim_end(), content);
        entry.1 = combined.into();
    } else {
        extracted.push((sys_path, content.to_string().into()));
    }
    theta_manifest::set_system_path(doc);
}

/// Read a cross-read file if it exists, returning the trimmed content.
/// `None` for missing files and for files whose content is whitespace-only.
pub(crate) fn read_cross_read_file(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }
    let content = fs_err::read_to_string(path).ok()?;
    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Emit a cross-read diagnostic hint.
pub(crate) fn cross_read_hint(rel_path: &str, source_harness: &str) -> Diagnostic {
    Diagnostic::hint(
        format!("[cross-read] {rel_path}"),
        format!(
            "imported {rel_path} - this content originates from {source_harness}. \
             subsequent round-trips may produce duplicates if the original file persists"
        ),
    )
}

/// Import a markdown agent file (frontmatter + body) as a subagent table entry.
/// Returns `(subagent_table, extracted_prompt_file)` or `None` if content is empty.
pub(crate) fn import_md_agent_as_subagent(
    stem: &str,
    content: &str,
) -> (toml_edit::Table, Option<CastFile>) {
    let parsed = parse_frontmatter(content);
    let mut sub = toml_edit::Table::new();
    sub["name"] = toml_edit::value(stem);
    let default_desc = format!("imported agent {stem}");
    let desc = parsed.get_str("description").unwrap_or(&default_desc);
    sub["description"] = toml_edit::value(desc);
    if let Some(model) = parsed.get_str("model") {
        sub["model"] = toml_edit::value(model);
    }
    let body = parsed.content.trim();
    let extracted = if body.is_empty() {
        None
    } else {
        let rel = theta_static::ThetaProjectLayout::subagent_prompt_rel(stem);
        sub["prompt_path"] = toml_edit::value(&rel);
        Some((PathBuf::from(rel), CastContent::Text(body.to_string())))
    };
    (sub, extracted)
}

/// Import a codex TOML agent file as a subagent table entry.
/// Returns `Ok((subagent_table, extracted_prompt_file))` or `Err` if parse fails.
pub(crate) fn import_toml_agent_as_subagent(
    stem: &str,
    content: &str,
    source_label: &str,
) -> Result<(toml_edit::Table, Option<CastFile>)> {
    let val: toml_edit::DocumentMut = content
        .parse()
        .with_context(|| format!("failed to parse {source_label} as TOML"))?;

    let mut sub = toml_edit::Table::new();
    let name = val.get("name").and_then(|v| v.as_str()).unwrap_or(stem);
    sub["name"] = toml_edit::value(name);
    let default_desc = format!("imported from {source_label}");
    let desc = val
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_desc);
    sub["description"] = toml_edit::value(desc);
    if let Some(model) = val.get("model").and_then(|v| v.as_str()) {
        sub["model"] = toml_edit::value(model);
    }
    let extracted = val
        .get("developer_instructions")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|instructions| {
            let rel = theta_static::ThetaProjectLayout::subagent_prompt_rel(stem);
            sub["prompt_path"] = toml_edit::value(&rel);
            (
                PathBuf::from(rel),
                CastContent::Text(instructions.trim().to_string()),
            )
        });
    Ok((sub, extracted))
}

/// Emit `Diagnostic::warn` for every rule whose `apply` mode the target harness
/// cannot represent natively. Cursor is the only harness with native
/// `model-decision` / `manual` semantics; everywhere else those modes collapse
/// to "always-injected" on cast, which is lossy.
pub(crate) fn collect_lossy_apply_warnings(
    manifest: &ThetaManifest,
    harness_label: &str,
) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let Some(ref instr) = manifest.instructions else {
        return diags;
    };
    let Some(ref rules) = instr.rules else {
        return diags;
    };
    for (name, rule) in rules {
        let mode = match rule.apply {
            theta_schema::ApplyMode::ModelDecision => "model-decision",
            theta_schema::ApplyMode::Manual => "manual",
            _ => continue,
        };
        diags.push(Diagnostic::warn(
            format!("[instructions.rules.{name}]"),
            format!(
                "rule \"{name}\" uses apply = \"{mode}\" which {harness_label} cannot represent natively - rule will be injected unconditionally on cast"
            ),
        ));
    }
    diags
}

/// Warn for every tool value containing `${env:NAME}` on harnesses that do
/// not resolve theta's `${env:...}` convention natively. The guidance text
/// is owned by `HarnessTarget::secret_placeholder_note()`.
pub(crate) fn collect_env_placeholder_warnings(
    manifest: &ThetaManifest,
    target: theta_harness::HarnessTarget,
) -> Vec<Diagnostic> {
    let Some(note) = target.secret_placeholder_note() else {
        return Vec::new();
    };
    let Some(ref tools) = manifest.tools else {
        return Vec::new();
    };
    let mut diags = Vec::new();
    for (name, tool) in tools.iter().filter(|(_, t)| t.enabled) {
        let ctx = format!("[tools.{name}]");
        let env_kv = tool.env.iter().flatten().map(|(k, v)| ("env", k, v));
        let hdr_kv = tool
            .headers
            .iter()
            .flatten()
            .map(|(k, v)| ("headers", k, v));
        for (section, key, value) in env_kv.chain(hdr_kv) {
            if value.contains("${env:") {
                diags.push(Diagnostic::warn(&ctx, format!("{section}.{key}: {note}")));
            }
        }
    }
    diags
}

#[cfg(test)]
mod tests;
