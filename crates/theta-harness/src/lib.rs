//! Harness identity, layout, and config types.

pub mod layout;

use strum::{Display, EnumIter, EnumString, IntoEnumIterator, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
#[non_exhaustive]
pub enum HarnessTarget {
    ClaudeCode,
    CodexCli,
    Copilot,
    Cursor,
}

impl HarnessTarget {
    pub fn as_str(&self) -> &'static str {
        (*self).into()
    }

    pub fn all() -> impl Iterator<Item = Self> {
        Self::iter()
    }

    pub fn toml_key(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude_code",
            Self::CodexCli => "codex",
            Self::Copilot => "github_copilot",
            Self::Cursor => "cursor",
        }
    }

    /// User-home directory name where this harness stores hand-curated global
    /// config (e.g. `~/.claude/CLAUDE.md`). `theta cast` MUST refuse to write
    /// into these directories without `--force` to avoid clobbering user state.
    ///
    /// Copilot uses VS Code's `~/.vscode/` rather than a Copilot-specific home
    /// dir — guarding the shared editor directory protects more than just
    /// Copilot config but is the right boundary.
    pub fn user_home_dir(&self) -> &'static str {
        match self {
            Self::ClaudeCode => ".claude",
            Self::CodexCli => ".codex",
            Self::Cursor => ".cursor",
            Self::Copilot => ".vscode",
        }
    }

    /// Per-harness guidance shown when a tool value contains theta's
    /// `${env:NAME}` convention. `None` means the harness resolves the
    /// convention natively and no warning is needed (Cursor).
    pub fn secret_placeholder_note(&self) -> Option<&'static str> {
        match self {
            Self::Cursor => None,
            Self::Copilot => Some(
                "GitHub Copilot does not resolve `${env:...}` - use `${input:NAME}` and declare a matching entry under `[harness.github_copilot.mcp_input_variables]`",
            ),
            Self::CodexCli => Some(
                "Codex CLI has no string interpolation for `${env:...}` - forward host env via `env_vars = [\"NAME\"]` under `[harness.codex.tool.<name>]`, or use `bearer_token_env_var = \"NAME\"` for HTTP Authorization",
            ),
            Self::ClaudeCode => Some(
                "Claude Code does not resolve `${env:...}` in `.mcp.json` - the literal string will reach the MCP server; resolve secrets out-of-band",
            ),
        }
    }
}
