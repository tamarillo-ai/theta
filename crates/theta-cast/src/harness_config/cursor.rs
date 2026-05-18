//! Cursor harness config — typed fields for `.cursor/hooks.json` and rule imports.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Cursor typed config — fields that cast reads/transforms.
/// Only fields that have both a cast-to handler AND an import path are typed here.
/// Everything else falls through to `extra` for lossless passthrough.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CursorConfig {
    /// Semver range constraint for Cursor version. Cast warns if outside range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Lifecycle hooks — contents of `.cursor/hooks.json`.
    /// Round-trips to `.cursor/hooks.json` on cast.
    /// ref: <https://cursor.com/docs/hooks>
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,

    /// Per-server MCP extras not modeled in theta `[tools]`, keyed by server name.
    /// Includes fields like `auth`, `envFile`, `type` that theta doesn't own.
    /// Merged into `.cursor/mcp.json` servers on cast; theta-typed fields win.
    /// ref: <https://cursor.com/docs/mcp>
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp_extras: BTreeMap<String, serde_json::Value>,

    /// Per-subagent extras keyed by subagent name.
    /// Stores Cursor-specific frontmatter fields like `readonly` and `is_background`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub subagent: BTreeMap<String, serde_json::Value>,

    /// All unrecognized fields pass through untouched.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl super::HasVersionConstraint for CursorConfig {
    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    fn harness_name(&self) -> &'static str {
        theta_harness::HarnessTarget::Cursor.toml_key()
    }
    fn detect_version(&self) -> Option<String> {
        crate::harnesses::version::detect_cursor_version()
    }
}
