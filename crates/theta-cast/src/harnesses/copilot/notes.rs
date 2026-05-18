//! Known limitations and clarifying notes for the copilot harness.

const SURFACE_MAP: &str = "\
cast (theta --> harness):
  identity + system prompt --> .github/copilot-instructions.md
  per-rule                 --> .github/instructions/<name>.instructions.md
  [skills]                 --> .github/skills/<name>/SKILL.md
  [tools]                  --> .vscode/mcp.json  ({\"servers\": {...}})
  [[subagents]]            --> .github/agents/<name>.agent.md
  harness config           --> .vscode/settings.json

import (harness --> theta):
  .github/copilot-instructions.md          --> identity + system prompt
  .github/instructions/**/*.instructions.md --> [instructions.rules]
  .github/skills/*/SKILL.md                --> [skills]
  .vscode/mcp.json servers                 --> [tools]
  .github/agents/*.agent.md                --> [[subagents]]
  .vscode/settings.json                    --> [harness.github_copilot]
  .github/hooks/*.json                     --> [harness.github_copilot].hooks
";

const REFS: &str = "\
ref: https://code.visualstudio.com/docs/copilot/customization/custom-instructions
ref: https://code.visualstudio.com/docs/copilot/chat/mcp-servers
ref: https://code.visualstudio.com/docs/copilot/customization/agent-skills
ref: https://code.visualstudio.com/docs/copilot/customization/custom-agents
ref: https://code.visualstudio.com/docs/copilot/reference/mcp-configuration
ref: https://code.visualstudio.com/docs/copilot/customization/hooks
";

const IMPORT_SETTINGS_PREFIXES: &str = "\
  settings prefixes    import grabs `github.copilot.*` and `chat.*` keys.
                       this includes agent-specific settings (chat.agent.*,
                       chat.tools.*, chat.mcp.*) AND general chat UI settings
                       (chat.editor.*, chat.fontSize, etc.). all are stored
                       in [harness.github_copilot] and round-trip verbatim.
                       non-copilot keys (editor.*, files.*, search.*) are NOT
                       imported but are preserved by the merge on cast.
";

const IMPORT_HOOKS: &str = "\
  hook consolidation   multiple .github/hooks/*.json files are consolidated
                       into [harness.github_copilot].hooks. original file
                       grouping is not preserved.
";

const IMPORT_YAML_COMMENTS: &str = "\
  YAML comments        # comments inside frontmatter are stripped (YAML spec).
";

const IMPORT_JSONC_COMMENTS: &str = "\
  JSONC comments       // comments in settings.json and mcp.json are stripped
                       during JSONC --> JSON parsing.
";

const IMPORT_CRLF: &str = "\
  CRLF normalization   Windows CRLF line endings normalize to LF on import.
";

const IMPORT_DISCOVERY_PATHS: &str = "\
  discovery paths      VS Code settings like chat.agentFilesLocations,
                       chat.instructionsFilesLocations, chat.hookFilesLocations,
                       chat.agentSkillsLocations, and chat.promptFilesLocations
                       override where VS Code looks for config files. theta
                       does NOT follow these paths -- import always reads from
                       the fixed well-known locations (.github/agents/,
                       .github/instructions/, etc.). the discovery settings
                       round-trip as opaque data but do not change theta's
                       import behavior.
";

const CAST_TRAILING_NEWLINE: &str = "\
  trailing newline     cast output always ends with \\n (POSIX). files that
                       originally lacked a trailing newline will differ.
";

const CAST_SUBAGENT_FILENAMES: &str = "\
  subagent filenames   cast uses kebab_case(name) for agent filenames.
                       original filename stem is not preserved.
";

const CAST_AGENT_NAME_ADDED: &str = "\
  agent name added     cast always emits `name:` in agent frontmatter.
                       agents that originally had no `name:` field will
                       have it added. VS Code derives name from filename
                       so this is functionally harmless.
";

const CAST_MCP_TYPE_ADDED: &str = "\
  mcp type added       cast may add `\"type\": \"stdio\"` to MCP server
                       configs when inferred from the `command` field.
";

const CAST_HOOKS_FILE: &str = "\
  hook output file     hooks are written to .github/hooks/theta-hooks.json.
                       if original hook files still exist alongside it,
                       VS Code merges all *.json and hooks may fire twice.
                       remove originals after importing into theta.
";

const CAST_SETTINGS_REORDER: &str = "\
  settings reorder     .vscode/settings.json keys may be reordered
                       alphabetically by serde_json.
";

const RT_YAML_QUOTE_STYLE: &str = "\
  YAML quote style     frontmatter values are re-serialized; double-quoted
                       strings may become single-quoted or unquoted.
";

const RT_YAML_INLINE_ARRAYS: &str = "\
  YAML inline arrays   tools: ['a', 'b'] may become a block sequence.
";

const RT_FRONTMATTER_ORDER: &str = "\
  frontmatter order    YAML key ordering is not preserved on round-trip.
";

/// Assemble notes from individual constants at runtime.
pub(crate) fn import_notes() -> String {
    format!(
        "GitHub Copilot -- import notes (theta cast from copilot)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         import-specific:\n\
         {IMPORT_SETTINGS_PREFIXES}\
         {IMPORT_DISCOVERY_PATHS}\
         {IMPORT_HOOKS}\
         {IMPORT_YAML_COMMENTS}\
         {IMPORT_JSONC_COMMENTS}\
         {IMPORT_CRLF}\n\
         round-trip:\n\
         {RT_YAML_QUOTE_STYLE}\
         {RT_YAML_INLINE_ARRAYS}\
         {RT_FRONTMATTER_ORDER}"
    )
}

pub(crate) fn cast_notes() -> String {
    format!(
        "GitHub Copilot -- cast notes (theta cast to copilot)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         cast-specific:\n\
         {CAST_TRAILING_NEWLINE}\
         {CAST_SUBAGENT_FILENAMES}\
         {CAST_AGENT_NAME_ADDED}\
         {CAST_MCP_TYPE_ADDED}\
         {CAST_HOOKS_FILE}\
         {CAST_SETTINGS_REORDER}\n\
         round-trip:\n\
         {RT_YAML_QUOTE_STYLE}\
         {RT_YAML_INLINE_ARRAYS}\
         {RT_FRONTMATTER_ORDER}"
    )
}
