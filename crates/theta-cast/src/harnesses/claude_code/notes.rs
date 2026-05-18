//! Known limitations and clarifying notes for the claude-code harness.

const SURFACE_MAP: &str = "\
cast (theta --> harness):
  identity + system prompt --> CLAUDE.md (opaque body, no synthetic header)
  per-rule                 --> .claude/rules/<name>.md (subdirs preserved)
  [skills]                 --> .claude/skills/<name>/SKILL.md (+ supporting files)
  [tools] + [harness.claude_code.tool.<name>] --> .mcp.json ({\"mcpServers\": {...}})
  [[subagents]] + [harness.claude_code.subagent.<slug>] --> .claude/agents/<name>.md
  [harness.claude_code]    --> .claude/settings.json (hooks/permissions/sandbox/...)

import (harness --> theta):
  CLAUDE.md (or .claude/CLAUDE.md)         --> [instructions].system (opaque)
  .claude/rules/**/*.md                    --> [instructions.rules.<qualified/name>]
  .claude/skills/*/SKILL.md                --> [skills] (byte-for-byte dir copy)
  .mcp.json mcpServers                     --> [tools] + [harness.claude_code.tool.<name>]
  .claude/agents/*.md                      --> [[subagents]] + [harness.claude_code.subagent.<slug>]
  .claude/settings.json                    --> [harness.claude_code] (lossless passthrough)
  AGENTS.md (opt-in via --cross-read)      --> appended to [instructions].system

out-of-scope (NOT imported, NOT touched):
  .claude/settings.local.json   (gitignored, personal)
  CLAUDE.local.md               (gitignored, personal)
  ~/.claude/**                  (user-level)
";

const REFS: &str = "\
ref: https://code.claude.com/docs/en/memory     (CLAUDE.md, .claude/rules/, AGENTS.md)
ref: https://code.claude.com/docs/en/settings   (.claude/settings.json scopes, fields)
ref: https://code.claude.com/docs/en/mcp        (.mcp.json mcpServers, env-var expansion)
ref: https://code.claude.com/docs/en/hooks      (hooks live inside settings.json[\"hooks\"])
ref: https://code.claude.com/docs/en/skills     (.claude/skills/<name>/SKILL.md)
ref: https://code.claude.com/docs/en/sub-agents (.claude/agents/<name>.md frontmatter)
";

const IMPORT_ALT_CLAUDE_MD: &str = "\
  alternate CLAUDE.md  Claude reads BOTH ./CLAUDE.md AND ./.claude/CLAUDE.md.
                       import accepts the alternate location, but cast always
                       writes the canonical ./CLAUDE.md. repos using
                       .claude/CLAUDE.md will see the file relocate after
                       round-trip. a hint diagnostic is emitted on import.
                       ref: https://code.claude.com/docs/en/memory#choose-where-to-put-claude-md-files
";

const IMPORT_AGENTS_MD_OPT_IN: &str = "\
  AGENTS.md cross-read Claude reads AGENTS.md only when CLAUDE.md imports it
                       via `@AGENTS.md`. theta does NOT follow @-imports.
                       AGENTS.md is read only when --cross-read is passed,
                       and its content is appended to [instructions].system.
                       without --cross-read, AGENTS.md is left untouched.
                       ref: https://code.claude.com/docs/en/memory#agents-md
";

const IMPORT_SKILL_PASSTHROUGH: &str = "\
  SKILL.md passthrough .claude/skills/*/SKILL.md and all sibling files are
                       copied byte-for-byte. theta does NOT parse SKILL.md
                       frontmatter, so claude-specific fields like
                       `disable-model-invocation`, `allowed-tools`, `paths`,
                       `context: fork`, `model:` etc. round-trip losslessly.
                       ref: https://code.claude.com/docs/en/skills
";

const IMPORT_SUBAGENT_EXTRAS: &str = "\
  subagent extras      every frontmatter field not in
                       {name, description, model, tools, prompt, skills}
                       lands in [harness.claude_code.subagent.<slug>]
                       verbatim. preserved fields include: maxTurns,
                       permissionMode, mcpServers, hooks, memory, background,
                       effort, isolation, color, initialPrompt, disallowedTools.
                       ref: https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields
";

const IMPORT_SUBAGENT_TOOLS_FORM: &str = "\
  subagent tools form  Claude accepts `tools:` as a YAML sequence or as a
                       comma-separated string. import handles both and
                       normalizes to a TOML array. cast emits the canonical
                       inline comma-separated form.
                       ref: https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields
";

const IMPORT_SUBAGENT_OPTIONAL_DESC: &str = "\
  optional description Claude marks `description:` as recommended but does
                       not require it. theta models it as Option<String>:
                       agents without `description:` import without one and
                       cast back without one. no synthetic placeholder.
";

const IMPORT_MCP_TYPED_VS_EXTRAS: &str = "\
  mcp typed vs extras  .mcp.json `mcpServers.<name>` is split: the typed keys
                       (type, command, args, env, url, headers) go to [tools];
                       everything else (oauth, headersHelper, alwaysLoad, ...)
                       goes to [harness.claude_code.tool.<name>] and round-trips
                       on cast. on import, theta infers `type = stdio` when
                       `command` is set and `type = http` when only `url` is
                       set, so the typed [tools] entry is well-formed even when
                       the source omitted `type`. `streamable-http` is accepted
                       as an alias for `http` per the Claude docs.
                       ref: https://code.claude.com/docs/en/mcp
";

const IMPORT_SETTINGS_PASSTHROUGH: &str = "\
  settings passthrough every top-level key in .claude/settings.json passes
                       through to [harness.claude_code] verbatim with its
                       native camelCase name (hooks, permissions, sandbox,
                       enabledPlugins, autoMode, viewMode, attribution,
                       worktree, env, statusLine, ...). theta does NOT
                       transform sandbox profiles, hook commands, or
                       permission patterns.
                       ref: https://code.claude.com/docs/en/settings
";

const IMPORT_RULES_RECURSIVE: &str = "\
  recursive rules      .claude/rules/ is walked recursively. subdirectories
                       become path-qualified rule names, e.g.
                       .claude/rules/frontend/api-design.md becomes
                       [instructions.rules.\"frontend/api-design\"].
                       Claude itself scans the dir recursively, so this is
                       semantically faithful.
                       ref: https://code.claude.com/docs/en/memory#organize-rules-with-claude-rules
";

const IMPORT_RULES_NO_FRONTMATTER_FIELDS: &str = "\
  rule frontmatter     Claude rule files have no documented frontmatter
                       schema beyond paths. only `paths:` is recognized; if
                       present it maps to apply = \"glob\" with apply_to set.
                       any other frontmatter keys are dropped on import.
";

const IMPORT_JSONC_COMMENTS: &str = "\
  JSONC comments       // comments in settings.json and .mcp.json are
                       stripped during JSONC --> JSON parsing.
";

const IMPORT_YAML_COMMENTS: &str = "\
  YAML comments        # comments inside agent frontmatter are stripped
                       (YAML spec).
";

const IMPORT_CRLF: &str = "\
  CRLF normalization   Windows CRLF line endings normalize to LF on import.
";

const CAST_CANONICAL_CLAUDE_MD: &str = "\
  canonical CLAUDE.md  cast always writes ./CLAUDE.md at the project root,
                       even when the source used .claude/CLAUDE.md. the old
                       file is NOT deleted by cast - remove it manually after
                       casting, or Claude will load both.
                       ref: https://code.claude.com/docs/en/memory#choose-where-to-put-claude-md-files
";

const CAST_AGENT_FILENAMES: &str = "\
  subagent filenames   cast uses kebab_case(name) for agent filenames. the
                       original filename stem is NOT preserved. extras lookup
                       tries both the kebab slug AND the original name to
                       tolerate manifests authored either way.
";

const CAST_AGENT_NAME_ADDED: &str = "\
  agent name added     cast always emits `name:` in agent frontmatter, even
                       though Claude falls back to the filename. this is a
                       documented round-trip normalization: agents that
                       originally had no `name:` field will have it added.
                       ref: https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields
";

const CAST_AGENT_FRONTMATTER_GATE: &str = "\
  no-frontmatter case  agents with no theta-typed fields and no extras emit
                       WITHOUT frontmatter, matching imports that had none.
                       this avoids fabricating a `name:`-only frontmatter
                       block on every cast.
";

const CAST_AGENT_TOOLS_FORM: &str = "\
  tools comma form     cast emits `tools:` as a single comma-separated YAML
                       string (the canonical inline form). repos that
                       originally used a YAML block sequence will see this
                       as a format change. semantically equivalent.
                       ref: https://code.claude.com/docs/en/sub-agents#supported-frontmatter-fields
";

const CAST_MCP_TYPE_ADDED: &str = "\
  mcp type added       cast emits `\"type\": \"stdio\"` when [tools.<name>]
                       defines a command, and `\"type\": \"http\"` when it
                       defines a url. repos that omitted `type` (relying on
                       Claude's inference) will see it added.
                       ref: https://code.claude.com/docs/en/mcp#configure-mcp-servers
";

const CAST_RECOMMENDED_MAX_LINES: &str = "\
  recommended length   cast emits a hint when CLAUDE.md exceeds 200 lines.
                       Claude itself does not enforce a limit, but Anthropic
                       recommends shorter memory files for better adherence.
                       ref: https://code.claude.com/docs/en/memory#write-effective-instructions
";

const CAST_SETTINGS_REORDER: &str = "\
  settings reorder     .claude/settings.json keys may be reordered
                       alphabetically by serde_json on cast. hook command
                       arrays and permission lists keep their internal order.
";

const CAST_MCP_REORDER: &str = "\
  mcp reorder          .mcp.json keys (servers and per-server fields) are
                       serialized via serde_json::to_string_pretty and may
                       be reordered relative to the source. server identity
                       and field values are preserved.
";

const CAST_TRAILING_NEWLINE: &str = "\
  trailing newline     cast output always ends with \\n (POSIX). files that
                       originally lacked a trailing newline will differ.
";

const RT_YAML_QUOTE_STYLE: &str = "\
  YAML quote style     frontmatter values are re-serialized; double-quoted
                       strings may become single-quoted or unquoted.
";

const RT_YAML_KEY_ORDER: &str = "\
  frontmatter order    YAML key ordering in agent frontmatter is not
                       preserved on round-trip. theta-typed keys
                       (name, description, model, tools, skills)
                       are emitted in a fixed order after any extras.
";

const RT_JSON_KEY_ORDER: &str = "\
  JSON key order       .mcp.json and .claude/settings.json key ordering is
                       not guaranteed to match the source after round-trip.
";

const NA_LOCAL_OVERRIDES: &str = "\
  .local. overrides    .claude/settings.local.json and CLAUDE.local.md are
                       gitignored, machine-local files. theta does not
                       import or cast them - leave them in place.
                       ref: https://code.claude.com/docs/en/settings#settings-files
";

const NA_USER_LEVEL: &str = "\
  ~/.claude/**         user-level Claude config (~/.claude/CLAUDE.md,
                       ~/.claude/settings.json, ~/.claude/agents/, ...) is
                       per-user state and out of scope for theta, which is
                       project-scoped.
                       ref: https://code.claude.com/docs/en/settings#settings-files
";

const NA_AT_IMPORTS: &str = "\
  @-imports            Claude supports `@path/to/file.md` imports inside
                       CLAUDE.md to compose memory across files. theta does
                       NOT resolve @-imports - the directive round-trips as
                       literal text in the system prompt body. consumers
                       relying on @-import expansion must run Claude itself.
                       ref: https://code.claude.com/docs/en/memory#use-imports-to-compose-memory
";

const NA_OUTPUT_STYLES: &str = "\
  output styles        Claude's output-style mechanism (the `outputStyle`
                       setting selecting a custom response format) has no
                       theta-spec counterpart and is preserved opaquely in
                       [harness.claude_code] when it appears in settings.json.
                       ref: https://code.claude.com/docs/en/settings
";

/// Assemble import notes from individual constants.
pub(crate) fn import_notes() -> String {
    format!(
        "Claude Code -- import notes (theta cast from claude-code)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         import-specific:\n\
         {IMPORT_ALT_CLAUDE_MD}\
         {IMPORT_AGENTS_MD_OPT_IN}\
         {IMPORT_RULES_RECURSIVE}\
         {IMPORT_RULES_NO_FRONTMATTER_FIELDS}\
         {IMPORT_SKILL_PASSTHROUGH}\
         {IMPORT_SUBAGENT_EXTRAS}\
         {IMPORT_SUBAGENT_TOOLS_FORM}\
         {IMPORT_SUBAGENT_OPTIONAL_DESC}\
         {IMPORT_MCP_TYPED_VS_EXTRAS}\
         {IMPORT_SETTINGS_PASSTHROUGH}\
         {IMPORT_JSONC_COMMENTS}\
         {IMPORT_YAML_COMMENTS}\
         {IMPORT_CRLF}\n\
         round-trip:\n\
         {RT_YAML_QUOTE_STYLE}\
         {RT_YAML_KEY_ORDER}\
         {RT_JSON_KEY_ORDER}\n\
         not applicable:\n\
         {NA_LOCAL_OVERRIDES}\
         {NA_USER_LEVEL}\
         {NA_AT_IMPORTS}\
         {NA_OUTPUT_STYLES}"
    )
}

/// Assemble cast notes from individual constants.
pub(crate) fn cast_notes() -> String {
    format!(
        "Claude Code -- cast notes (theta cast to claude-code)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         cast-specific:\n\
         {CAST_CANONICAL_CLAUDE_MD}\
         {CAST_AGENT_FILENAMES}\
         {CAST_AGENT_NAME_ADDED}\
         {CAST_AGENT_FRONTMATTER_GATE}\
         {CAST_AGENT_TOOLS_FORM}\
         {CAST_MCP_TYPE_ADDED}\
         {CAST_RECOMMENDED_MAX_LINES}\
         {CAST_SETTINGS_REORDER}\
         {CAST_MCP_REORDER}\
         {CAST_TRAILING_NEWLINE}\n\
         round-trip:\n\
         {RT_YAML_QUOTE_STYLE}\
         {RT_YAML_KEY_ORDER}\
         {RT_JSON_KEY_ORDER}\n\
         not applicable:\n\
         {NA_LOCAL_OVERRIDES}\
         {NA_USER_LEVEL}\
         {NA_AT_IMPORTS}\
         {NA_OUTPUT_STYLES}"
    )
}
