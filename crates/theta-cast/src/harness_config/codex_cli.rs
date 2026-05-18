//! Codex CLI harness config — typed fields for `.codex/config.toml`.
//!
//! Almost everything in `.codex/config.toml` is opaque passthrough. theta does
//! NOT model sandbox / approval / features / `model_providers` / profiles / etc
//! as typed structs because cast does not transform any of those — they
//! round-trip byte-for-byte through the `extra` flatten map with their native
//! `snake_case` keys preserved.
//!
//! Only fields theta actively consumes need typed slots:
//! - `version`: `HasVersionConstraint` for installed-version range checks
//! - `tool`: per-server MCP extras merged into `[mcp_servers.<name>]` on cast
//! - `subagent`: per-agent extras merged into `.codex/agents/<slug>.toml` on cast
//!
//! See `scratch/codex_cast_tests/CODEX_DEEP_DIVE.md` (§6.1, decision D3).
//! ref: <https://developers.openai.com/codex/config-reference>

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodexCliConfig {
    /// Semver range constraint, e.g. `">=0.1.0"`. Cast warns if installed
    /// version is outside range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Per-server MCP extras not modeled in theta `[tools]`, keyed by server
    /// name. Merged into `[mcp_servers.<name>]` on cast; theta-typed fields
    /// win. Stores fields like `env_vars`, `cwd`, `bearer_token_env_var`,
    /// `env_http_headers`, `oauth_resource`, `scopes`,
    /// `experimental_environment`, `startup_timeout_sec`, `tool_timeout_sec`,
    /// `enabled`, `required`, `enabled_tools`, `disabled_tools`.
    /// ref: <https://developers.openai.com/codex/mcp>
    /// ref: <https://developers.openai.com/codex/config-reference>
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tool: BTreeMap<String, serde_json::Value>,

    /// Per-subagent extras not modeled in theta `[[subagents]]`, keyed by
    /// filename stem (kebab slug). Merged into `.codex/agents/<slug>.toml`
    /// on cast; theta-typed fields (`name`, `description`,
    /// `developer_instructions`, `model`) win. A codex subagent file is a
    /// full config-layer per the docs and may contain any supported
    /// `config.toml` key: `nickname_candidates`, `sandbox_mode`,
    /// `model_reasoning_effort`, nested `[mcp_servers.*]`, `[permissions.*]`,
    /// `[skills.config]`, `[features]` overrides, etc.
    /// ref: <https://developers.openai.com/codex/subagents#custom-agent-file-schema>
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub subagent: BTreeMap<String, serde_json::Value>,

    /// Every other top-level key from `.codex/config.toml` (`sandbox_mode`,
    /// `approval_policy`, `web_search`, `personality`, `model_reasoning_effort`,
    /// `[features]`, `[sandbox_workspace_write]`, `[model_providers.*]`,
    /// `[profiles.*]`, `[agents]`, `[tui]`, `[otel]`, `[shell_environment_policy]`,
    /// `[permissions.*]`, `hooks`, and any future addition) passes through
    /// unchanged with its original `snake_case` key.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl super::HasVersionConstraint for CodexCliConfig {
    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    fn harness_name(&self) -> &'static str {
        theta_harness::HarnessTarget::CodexCli.toml_key()
    }
    fn detect_version(&self) -> Option<String> {
        crate::harnesses::version::detect_codex_cli_version()
    }
}
