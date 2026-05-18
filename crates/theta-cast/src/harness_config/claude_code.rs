//! Claude Code harness config — typed fields for `.claude/settings.json`.
//!
//! Almost everything in `.claude/settings.json` is opaque passthrough. theta does
//! NOT model sandbox/hooks/permissions/etc as typed structs because cast does
//! not transform any of those fields — they round-trip byte-for-byte through
//! the `extra` flatten map with their native camelCase keys preserved.
//!
//! Only fields theta actively consumes need typed slots:
//! - `version`: `HasVersionConstraint` for installed-version range checks
//! - `tool`: per-server MCP extras merged into `.mcp.json` on cast
//! - `subagent`: per-agent frontmatter extras merged into `.claude/agents/` on cast
//!
//! See `scratch/claude_cast_tests/SURFACES.md` (settings.json scope decision).
//! ref: <https://code.claude.com/docs/en/settings#available-settings>

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    /// Semver range constraint, e.g. `">=1.0.32"`. Cast warns if installed version is outside range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Per-server MCP extras not modeled in theta `[tools]`, keyed by server name.
    /// Merged into `.mcp.json` `mcpServers.<name>` on cast; theta-typed fields win.
    /// Stores fields like `type`, `oauth`, `headersHelper`, `alwaysLoad` that
    /// Claude Code understands but theta does not.
    /// ref: <https://code.claude.com/docs/en/mcp#environment-variable-expansion-in-mcp-json>
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tool: BTreeMap<String, serde_json::Value>,

    /// Per-subagent frontmatter extras not modeled in theta `[[subagents]]`,
    /// keyed by filename stem (kebab slug). Merged into `.claude/agents/<name>.md`
    /// frontmatter on cast; theta-typed fields win. Stores fields like
    /// `permissionMode`, `mcpServers`, `hooks`, `memory`, `background`, `effort`,
    /// `isolation`, `color`, `initialPrompt`, `disallowedTools`.
    /// ref: <https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields>
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub subagent: BTreeMap<String, serde_json::Value>,

    /// Every other top-level key from `.claude/settings.json` (sandbox, hooks,
    /// permissions, enabledPlugins, autoMode, viewMode, attribution, worktree,
    /// statusLine, env, claudeMdExcludes, autoMemory*, skillOverrides, and any
    /// future addition) passes through unchanged with its original camelCase key.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl super::HasVersionConstraint for ClaudeCodeConfig {
    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    fn harness_name(&self) -> &'static str {
        theta_harness::HarnessTarget::ClaudeCode.toml_key()
    }
    fn detect_version(&self) -> Option<String> {
        crate::harnesses::version::detect_claude_code_version()
    }
}
