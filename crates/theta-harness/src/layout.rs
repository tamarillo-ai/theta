//! Harness output layouts — typed file structure for cast to/from.
//!
//! Each harness declares its complete file surface as a layout struct.
//! Pure path resolution, no IO. Used by both casters (write) and
//! importers (read) so the file structure is defined once.
//!
//! The [`HarnessLayout`] trait captures the shared surface: system prompt,
//! skills, and subagents. Harness-specific paths (hooks, settings, config)
//! stay as inherent methods on the concrete structs.

use std::path::PathBuf;

/// Flatten a path-qualified rule name for harnesses with flat rule directories.
/// `"review/pr-review"` --> `"review-pr-review"`, simple names pass through unchanged.
pub fn flatten_rule_name(name: &str) -> String {
    name.replace('/', "-")
}

/// Shared file surface across all harnesses.
///
/// Each harness places system prompts, rules, skills, and subagents in
/// different directories — this trait unifies the path resolution so
/// common code can write generic helpers.
pub trait HarnessLayout {
    /// Primary system-prompt / instructions file (e.g. `CLAUDE.md`, `AGENTS.md`)
    fn system_prompt() -> PathBuf;

    /// Primary skills directory
    fn skills_dir() -> PathBuf;

    /// Subagents directory
    fn agents_dir() -> PathBuf;

    /// Individual subagent file
    fn agent(name: &str) -> PathBuf;
}

// Claude Code
// ref: https://code.claude.com/docs/en/memory       (CLAUDE.md, .claude/rules/)
// ref: https://code.claude.com/docs/en/settings     (.claude/settings.json)
// ref: https://code.claude.com/docs/en/mcp          (.mcp.json)
// ref: https://code.claude.com/docs/en/skills       (.claude/skills/)
// ref: https://code.claude.com/docs/en/agents       (.claude/agents/)

pub struct ClaudeCodeLayout;

impl HarnessLayout for ClaudeCodeLayout {
    fn system_prompt() -> PathBuf {
        PathBuf::from("CLAUDE.md")
    }

    fn skills_dir() -> PathBuf {
        PathBuf::from(".claude/skills")
    }

    fn agents_dir() -> PathBuf {
        PathBuf::from(".claude/agents")
    }

    fn agent(name: &str) -> PathBuf {
        PathBuf::from(format!(".claude/agents/{name}.md"))
    }
}

impl ClaudeCodeLayout {
    /// `.claude/rules/<name>.md` — preserves path-qualified subdirectories
    /// (e.g. `"frontend/api-design"` --> `.claude/rules/frontend/api-design.md`).
    /// Claude scans `.claude/rules/` recursively, so subdirectories are first-class.
    /// ref: <https://code.claude.com/docs/en/memory#organize-rules-with-claude-rules>
    pub fn rule(name: &str) -> PathBuf {
        PathBuf::from(format!(".claude/rules/{name}.md"))
    }

    /// Alternate system prompt location: Claude reads `./CLAUDE.md` AND `./.claude/CLAUDE.md`.
    /// ref: <https://code.claude.com/docs/en/memory#choose-where-to-put-claude-md-files>
    pub fn system_prompt_alt() -> PathBuf {
        PathBuf::from(".claude/CLAUDE.md")
    }

    pub fn rules_dir() -> PathBuf {
        PathBuf::from(".claude/rules")
    }

    pub fn settings() -> PathBuf {
        PathBuf::from(".claude/settings.json")
    }

    pub fn mcp() -> PathBuf {
        PathBuf::from(".mcp.json")
    }

    pub fn skill(name: &str) -> PathBuf {
        PathBuf::from(format!(".claude/skills/{name}/SKILL.md"))
    }

    /// Root key inside `.mcp.json` for Claude Code: `{"mcpServers": {...}}`.
    /// ref: <https://code.claude.com/docs/en/mcp#project-scope>
    pub const MCP_ROOT_KEY: &'static str = "mcpServers";

    /// Cross-read: always-on instruction files Claude discovers from other
    /// harness locations or shared conventions.
    ///
    /// Claude reads `AGENTS.md` only when `CLAUDE.md` imports it via `@AGENTS.md`,
    /// but in practice repositories often use `AGENTS.md` as the primary
    /// always-on instruction file shared between codex / copilot / claude.
    /// theta's `--cross-read` option concatenates these into the system prompt.
    ///
    /// ref: <https://code.claude.com/docs/en/memory#agents-md>
    pub const CROSS_READ_SYSTEM_PROMPT_FILES: &[&str] = &["AGENTS.md"];
}

// GitHub Copilot (VS Code)
// ref: https://code.visualstudio.com/docs/copilot/customization/custom-instructions (.github/copilot-instructions.md, .github/instructions/)
// ref: https://code.visualstudio.com/docs/copilot/chat/mcp-servers              (.vscode/mcp.json, "servers" key)
// ref: https://code.visualstudio.com/docs/copilot/customization/agent-skills     (.github/skills/)
// ref: https://code.visualstudio.com/docs/copilot/customization/custom-agents    (.github/agents/)
// ref: https://code.visualstudio.com/docs/copilot/reference/mcp-configuration    (mcp.json schema)

pub struct CopilotLayout;

impl HarnessLayout for CopilotLayout {
    fn system_prompt() -> PathBuf {
        PathBuf::from(".github/copilot-instructions.md")
    }

    fn skills_dir() -> PathBuf {
        PathBuf::from(".github/skills")
    }

    fn agents_dir() -> PathBuf {
        PathBuf::from(".github/agents")
    }

    fn agent(name: &str) -> PathBuf {
        PathBuf::from(format!(".github/agents/{name}.agent.md"))
    }
}

impl CopilotLayout {
    pub fn rule(name: &str) -> PathBuf {
        PathBuf::from(format!(".github/instructions/{name}.instructions.md"))
    }

    pub fn settings() -> PathBuf {
        PathBuf::from(".vscode/settings.json")
    }

    pub fn mcp() -> PathBuf {
        PathBuf::from(".vscode/mcp.json")
    }

    pub fn skill(name: &str) -> PathBuf {
        PathBuf::from(format!(".github/skills/{name}/SKILL.md"))
    }

    pub fn prompts_dir() -> PathBuf {
        PathBuf::from(".github/prompts")
    }

    pub fn hooks_dir() -> PathBuf {
        PathBuf::from(".github/hooks")
    }

    pub fn hooks_file() -> PathBuf {
        PathBuf::from(".github/hooks/theta-hooks.json")
    }

    /// Cross-read: always-on instruction files from other harnesses.
    /// ref: <https://code.visualstudio.com/docs/copilot/customization/custom-instructions>
    pub const CROSS_READ_SYSTEM_PROMPT_FILES: &[&str] = &[
        "AGENTS.md",
        "CLAUDE.md",
        ".claude/CLAUDE.md",
        "CLAUDE.local.md",
    ];

    /// Cross-read: claude-format rules directory.
    pub const CROSS_READ_CLAUDE_RULES_DIR: &str = ".claude/rules";
}

// Cursor
// ref: https://cursor.com/docs/rules                 (.cursor/rules/)
// ref: https://cursor.com/docs/mcp                   (.cursor/mcp.json, "mcpServers" key)
// ref: https://cursor.com/docs/hooks                 (.cursor/hooks.json)
// ref: https://cursor.com/docs/skills                (.cursor/skills/, .agents/skills/)

pub struct CursorLayout;

impl HarnessLayout for CursorLayout {
    fn system_prompt() -> PathBuf {
        PathBuf::from(".cursor/rules/system.md")
    }

    fn skills_dir() -> PathBuf {
        PathBuf::from(".cursor/skills")
    }

    fn agents_dir() -> PathBuf {
        PathBuf::from(".cursor/agents")
    }

    fn agent(name: &str) -> PathBuf {
        PathBuf::from(format!(".cursor/agents/{name}.md"))
    }
}

impl CursorLayout {
    pub fn rules_dir() -> PathBuf {
        PathBuf::from(".cursor/rules")
    }

    pub fn rule(name: &str) -> PathBuf {
        let flat = flatten_rule_name(name);
        PathBuf::from(format!(".cursor/rules/{flat}.mdc"))
    }

    pub fn hooks() -> PathBuf {
        PathBuf::from(".cursor/hooks.json")
    }

    pub fn mcp() -> PathBuf {
        PathBuf::from(".cursor/mcp.json")
    }

    pub fn skill(name: &str) -> PathBuf {
        PathBuf::from(format!(".cursor/skills/{name}/SKILL.md"))
    }

    /// Cross-agent skills (.agents/ convention).
    pub fn cross_agent_skills_dir() -> PathBuf {
        PathBuf::from(".agents/skills")
    }

    /// Cross-read: always-on instruction file from other harnesses.
    pub const CROSS_READ_AGENTS_MD: &str = "AGENTS.md";

    /// Cross-read: subagent directories from other harnesses (`dir_path`, `file_extension`).
    /// ref: <https://cursor.com/docs/subagents#file-locations>
    pub const CROSS_READ_AGENT_DIRS: &[(&str, &str)] =
        &[(".claude/agents", "md"), (".codex/agents", "toml")];
}

// Codex CLI
// ref: https://developers.openai.com/codex/guides/agents-md   (AGENTS.md)
// ref: https://developers.openai.com/codex/mcp               (.codex/config.toml [mcp_servers])
// ref: https://developers.openai.com/codex/config-advanced   (.codex/config.toml, hooks.json)
// ref: https://developers.openai.com/codex/skills            (.codex/skills/, .agents/skills/)
// ref: https://developers.openai.com/codex/subagents         (.codex/agents/<name>.toml)

pub struct CodexCliLayout;

impl HarnessLayout for CodexCliLayout {
    fn system_prompt() -> PathBuf {
        PathBuf::from("AGENTS.md")
    }

    fn skills_dir() -> PathBuf {
        PathBuf::from(".codex/skills")
    }

    fn agents_dir() -> PathBuf {
        PathBuf::from(".codex/agents")
    }

    fn agent(name: &str) -> PathBuf {
        PathBuf::from(format!(".codex/agents/{name}.toml"))
    }
}

impl CodexCliLayout {
    // file paths

    pub fn agents_md() -> PathBuf {
        <Self as HarnessLayout>::system_prompt()
    }

    pub fn config() -> PathBuf {
        PathBuf::from(".codex/config.toml")
    }

    pub fn hooks() -> PathBuf {
        PathBuf::from(".codex/hooks.json")
    }

    /// Codex-native skills (alias for trait's `skills_dir`).
    /// ref: <https://github.com/openai/codex/tree/main/.codex/skills>
    pub fn codex_skills_dir() -> PathBuf {
        <Self as HarnessLayout>::skills_dir()
    }

    pub fn skill(name: &str) -> PathBuf {
        PathBuf::from(format!(".codex/skills/{name}/SKILL.md"))
    }

    /// Cross-agent skills (.agents/ convention).
    /// ref: <https://developers.openai.com/codex/skills>
    pub fn cross_agent_skills_dir() -> PathBuf {
        PathBuf::from(".agents/skills")
    }

    /// Exec-policy rules (`.codex/rules/<name>.rules`, Starlark format).
    /// ref: <https://developers.openai.com/codex/rules>
    pub fn rules_dir() -> PathBuf {
        PathBuf::from(".codex/rules")
    }

    // on-disk key names (config.toml, agent TOMLs, hooks.json)

    /// Root key inside `.codex/config.toml` for MCP servers:
    /// `[mcp_servers.<name>]`.
    /// ref: <https://developers.openai.com/codex/mcp>
    pub const MCP_ROOT_KEY: &'static str = "mcp_servers";

    /// MCP HTTP headers key (`snake_case`; claude / vscode use `headers`).
    /// ref: <https://developers.openai.com/codex/config-reference>
    pub const MCP_HTTP_HEADERS_KEY: &'static str = "http_headers";

    /// Codex MCP key for `enabled = false` (servers registered but inactive).
    /// ref: <https://developers.openai.com/codex/mcp>
    pub const MCP_ENABLED_KEY: &'static str = "enabled";

    /// Per-server keys theta consumes as typed `[tools.<name>]` fields.
    /// Everything else round-trips through `[harness.codex.tool.<name>]`.
    /// ref: <https://developers.openai.com/codex/mcp>
    pub const TYPED_MCP_KEYS: &'static [&'static str] = &[
        theta_static::MCP_KEY_COMMAND,
        theta_static::MCP_KEY_ARGS,
        theta_static::MCP_KEY_URL,
        theta_static::MCP_KEY_ENV,
        Self::MCP_HTTP_HEADERS_KEY,
        Self::MCP_ENABLED_KEY,
    ];

    /// Subagent TOML file key names that theta carves out as typed
    /// `[[subagents]]` fields. Everything else round-trips through
    /// `[harness.codex.subagent.<slug>]`.
    /// ref: <https://developers.openai.com/codex/subagents#custom-agent-file-schema>
    pub const AGENT_KEY_NAME: &'static str = "name";
    pub const AGENT_KEY_DESCRIPTION: &'static str = "description";
    pub const AGENT_KEY_DEV_INSTRUCTIONS: &'static str = "developer_instructions";
    pub const AGENT_KEY_MODEL: &'static str = "model";

    pub const TYPED_AGENT_KEYS: &'static [&'static str] = &[
        Self::AGENT_KEY_NAME,
        Self::AGENT_KEY_DESCRIPTION,
        Self::AGENT_KEY_DEV_INSTRUCTIONS,
        Self::AGENT_KEY_MODEL,
    ];

    /// `config.toml` top-level key for hooks (mirrors the JSON file form).
    /// ref: <https://developers.openai.com/codex/hooks>
    pub const HOOKS_KEY: &'static str = "hooks";

    /// `[harness.codex]` knob: when `true`, cast emits skills to
    /// `.codex/skills/<name>/` (legacy codex-specific path) instead of the
    /// canonical `.agents/skills/<name>/`.
    pub const SPECIFIC_SKILLS_KEY: &'static str = "codex_specific_skills";
}
