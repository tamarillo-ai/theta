//! GitHub Copilot harness config — typed fields for `.agent.md` frontmatter and VS Code settings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// GitHub Copilot typed config — fields that cast reads/transforms.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CopilotConfig {
    /// Semver range constraint for the Copilot extension version. Cast warns if outside range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Lifecycle hooks — merged contents of all `.github/hooks/*.json` files.
    /// Round-trips to `.github/hooks/theta-hooks.json` on cast.
    /// See <https://code.visualstudio.com/docs/copilot/customization/hooks>.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<serde_json::Value>,

    /// Per-server MCP extras not modeled in theta `[tools]`, keyed by server name.
    /// Merged into `.vscode/mcp.json` servers on cast; theta-typed fields win.
    /// See <https://code.visualstudio.com/docs/copilot/reference/mcp-configuration>.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tool: BTreeMap<String, serde_json::Value>,

    /// MCP top-level `inputs` array — VS Code input-variable prompts.
    /// Round-trips verbatim to `.vscode/mcp.json` top-level `inputs`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_input_variables: Option<serde_json::Value>,

    /// Per-subagent frontmatter extras not modeled in theta `[[subagents]]`, keyed by agent name.
    /// Merged into `.github/agents/<name>.agent.md` frontmatter on cast; theta-typed fields win.
    /// See <https://code.visualstudio.com/docs/copilot/customization/custom-agents>.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub subagent: BTreeMap<String, serde_json::Value>,

    /// All unrecognized fields pass through untouched.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl super::HasVersionConstraint for CopilotConfig {
    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    fn harness_name(&self) -> &'static str {
        theta_harness::HarnessTarget::Copilot.toml_key()
    }
    fn detect_version(&self) -> Option<String> {
        crate::harnesses::version::detect_copilot_version()
    }
}
