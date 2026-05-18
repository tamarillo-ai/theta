// Claude Code caster
//
// cast (theta --> harness):
//   [instructions].system          --> CLAUDE.md (opaque body, no synthetic header)
//   [instructions.rules.<name>]    --> .claude/rules/<name>.md (recursive subdirs preserved)
//   [harness.claude_code]          --> .claude/settings.json (hooks under "hooks" key)
//   [tools] + [harness.claude_code.tool.<name>] --> .mcp.json (theta-typed wins, extras preserved)
//   [skills]                       --> .claude/skills/<name>/SKILL.md (+ supporting files)
//   [[subagents]] + [harness.claude_code.subagent.<slug>] --> .claude/agents/<name>.md
//
// import (harness --> theta):
//   CLAUDE.md (or .claude/CLAUDE.md) --> [instructions].system (opaque body)
//   .claude/rules/**/*.md            --> [instructions.rules.<path/qualified/name>] (recursive)
//   .claude/settings.json            --> [harness.claude_code] (lossless passthrough via `extra`)
//   .mcp.json mcpServers             --> [tools] + [harness.claude_code.tool.<name>] for extras
//   .claude/skills/*/SKILL.md        --> [skills] (byte-for-byte directory copy)
//   .claude/agents/*.md              --> [[subagents]] + [harness.claude_code.subagent.<slug>]
//
// out-of-scope (NOT imported, NOT touched):
//   .claude/settings.local.json   (gitignored, personal)
//   CLAUDE.local.md               (gitignored, personal)
//   ~/.claude/**                  (user-level)
//
// ref: https://code.claude.com/docs/en/memory     (CLAUDE.md, .claude/rules/, AGENTS.md import)
// ref: https://code.claude.com/docs/en/settings   (.claude/settings.json scopes, fields)
// ref: https://code.claude.com/docs/en/mcp        (.mcp.json mcpServers, env-var expansion)
// ref: https://code.claude.com/docs/en/hooks      (hooks live inside settings.json["hooks"])
// ref: https://code.claude.com/docs/en/skills     (.claude/skills/<name>/SKILL.md)
// ref: https://code.claude.com/docs/en/sub-agents (.claude/agents/<name>.md frontmatter fields)

mod cast;
mod import;
pub(crate) mod notes;

pub struct ClaudeCode;

#[cfg(test)]
mod tests;
