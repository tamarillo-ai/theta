//! Known limitations and clarifying notes for the cursor harness.

const SURFACE_MAP: &str = "\
cast (theta --> harness):
  identity + system prompt --> .cursor/rules/system.md   (alwaysApply: true)
  per-rule                 --> .cursor/rules/<name>.mdc
  [skills]                 --> .cursor/skills/<name>/SKILL.md
  [tools]                  --> .cursor/mcp.json           ({\"mcpServers\": {...}})
  [[subagents]]            --> .cursor/agents/<name>.md
  harness config           --> .cursor/hooks.json

import (harness --> theta):
  .cursor/rules/system.md                --> identity + system prompt
  .cursor/rules/*.{md,mdc}              --> [instructions.rules]
  .cursor/skills/*/SKILL.md             --> [skills]
  .agents/skills/*/SKILL.md             --> [skills] (cross-agent)
  .cursor/mcp.json mcpServers           --> [tools]
  .cursor/agents/*.md                   --> [[subagents]]
  .cursor/hooks.json                    --> [harness.cursor].hooks
";

const REFS: &str = "\
ref: https://cursor.com/docs/rules
ref: https://cursor.com/docs/mcp
ref: https://cursor.com/docs/hooks
ref: https://cursor.com/docs/skills
ref: https://cursor.com/docs/subagents
";

const IMPORT_INVALID_YAML: &str = "\
  invalid YAML         Cursor allows frontmatter that violates the YAML spec.
                       common violations:
                       - `globs: *.ts` - `*` is the c-alias indicator (§5.3)
                       - `description: text: more` - unquoted `: ` in values
                       when YAML parsing fails, theta imports the rule body
                       but drops frontmatter fields (description, globs,
                       alwaysApply). a warning is emitted per affected file.
                       fix: quote values containing `:` or starting with `*`.
                       ref: https://yaml.org/spec/1.2.2/ §5.3, §7.3.3
";

const IMPORT_YAML_COMMENTS: &str = "\
  YAML comments        # comments inside frontmatter are stripped (YAML spec).
";

const IMPORT_JSONC_COMMENTS: &str = "\
  JSONC comments       // comments in mcp.json and hooks.json are stripped
                       during JSONC --> JSON parsing.
";

const IMPORT_CRLF: &str = "\
  CRLF normalization   Windows CRLF line endings normalize to LF on import.
";

const IMPORT_EMPTY_FIELDS: &str = "\
  empty fields         frontmatter fields with YAML null values
                       (e.g. `description: `, `globs: `) are stripped.
                       YAML parses trailing-space values as null, and null
                       fields are not preserved in theta.toml.
";

const IMPORT_MCP_WRONG_KEY: &str = "\
  mcp root key         .cursor/mcp.json must use \"mcpServers\" as the root
                       key. files using \"servers\" or other keys are ignored
                       with a warning. ref: https://cursor.com/docs/mcp
";

const IMPORT_HOOKS_NULL: &str = "\
  hooks loop_limit     loop_limit: null means unlimited in Cursor, but TOML
                       has no null type -- key is omitted (defaults to 5).
                       use an explicit high number instead.
";

const IMPORT_LEGACY_CURSORRULES: &str = "\
  .cursorrules         legacy root-level .cursorrules file is not imported.
                       migrate to .cursor/rules/ for theta compatibility.
                       ref: https://cursor.com/docs/rules
";

const IMPORT_COMMANDS_DEPRECATED: &str = "\
  .cursor/commands/    commands are deprecated. Cursor replaced them with
                       skills (disable-model-invocation: true). use the
                       built-in /migrate-to-skills to convert. theta does
                       not import commands - they appear as UNMANAGED.
                       ref: https://cursor.com/docs/skills#migrating-rules-and-commands-to-skills
";

const CAST_TRAILING_NEWLINE: &str = "\
  trailing newline     cast output always ends with \\n (POSIX). files that
                       originally lacked a trailing newline will differ.
";

const CAST_AGENT_NAME_STRIPPED: &str = "\
  agent name           cast does not emit `name:` in agent frontmatter.
                       Cursor derives the agent name from the filename.
                       ref: https://cursor.com/docs/subagents#configuration-fields
";

const CAST_SYSTEM_MD: &str = "\
  system.md            cast emits .cursor/rules/system.md when the manifest
                       has a system prompt or non-default identity. repos
                       that had no system.md will see this as a new file.
";

const RT_YAML_QUOTE_STYLE: &str = "\
  YAML quote style     frontmatter values are re-serialized; double-quoted
                       strings may become single-quoted or unquoted.
";

const RT_YAML_KEY_ORDER: &str = "\
  frontmatter order    YAML key ordering is not preserved on round-trip.
";

const RT_GLOB_FORMAT: &str = "\
  glob format          comma-separated globs (e.g. `globs: *.ts, *.tsx`)
                       are normalized to YAML block sequences on round-trip.
                       both forms are semantically equivalent.
";

const RT_MCP_JSON_FORMAT: &str = "\
  mcp.json format      key ordering may change to alphabetical. indentation
                       normalized to 2-space pretty-print. JSONC comments
                       stripped.
";

const NA_PLUGIN_MANIFESTS: &str = "\
  plugin manifests     .cursor-plugin/plugin.json is a distribution wrapper.
                       theta imports individual components from canonical
                       paths, not the plugin manifest itself.
                       ref: https://cursor.com/docs/reference/plugins
";

const NA_PERMISSIONS: &str = "\
  permissions.json     ~/.cursor/permissions.json is user-level only.
                       theta does not manage user-level Cursor settings.
                       ref: https://cursor.com/docs/reference/permissions
";

const NA_TEAM_ENTERPRISE: &str = "\
  team/enterprise      Team Rules, Enterprise Hooks, and Team Marketplaces
                       are cloud-managed via the Cursor Dashboard. no file
                       representation in the project.
";

/// Assemble import notes from individual constants.
pub(crate) fn import_notes() -> String {
    format!(
        "Cursor -- import notes (theta cast from cursor)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         import-specific:\n\
         {IMPORT_INVALID_YAML}\
         {IMPORT_EMPTY_FIELDS}\
         {IMPORT_MCP_WRONG_KEY}\
         {IMPORT_HOOKS_NULL}\
         {IMPORT_LEGACY_CURSORRULES}\
         {IMPORT_COMMANDS_DEPRECATED}\
         {IMPORT_YAML_COMMENTS}\
         {IMPORT_JSONC_COMMENTS}\
         {IMPORT_CRLF}\n\
         round-trip:\n\
         {RT_YAML_QUOTE_STYLE}\
         {RT_YAML_KEY_ORDER}\
         {RT_GLOB_FORMAT}\
         {RT_MCP_JSON_FORMAT}\n\
         not applicable:\n\
         {NA_PLUGIN_MANIFESTS}\
         {NA_PERMISSIONS}\
         {NA_TEAM_ENTERPRISE}"
    )
}

/// Assemble cast notes from individual constants.
pub(crate) fn cast_notes() -> String {
    format!(
        "Cursor -- cast notes (theta cast to cursor)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         cast-specific:\n\
         {CAST_TRAILING_NEWLINE}\
         {CAST_AGENT_NAME_STRIPPED}\
         {CAST_SYSTEM_MD}\
         {IMPORT_COMMANDS_DEPRECATED}\n\
         round-trip:\n\
         {RT_YAML_QUOTE_STYLE}\
         {RT_YAML_KEY_ORDER}\
         {RT_GLOB_FORMAT}\
         {RT_MCP_JSON_FORMAT}\n\
         not applicable:\n\
         {NA_PLUGIN_MANIFESTS}\
         {NA_PERMISSIONS}\
         {NA_TEAM_ENTERPRISE}"
    )
}
