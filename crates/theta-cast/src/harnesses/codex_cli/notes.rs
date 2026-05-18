//! Known limitations and clarifying notes for the codex-cli harness.

const SURFACE_MAP: &str = "\
cast (theta --> harness):
  identity + system prompt + all rules --> AGENTS.md (concatenated, rules flattened)
  [tools] + [harness.codex.tool.<name>] --> .codex/config.toml [mcp_servers.<name>]
  [harness.codex]                      --> .codex/config.toml (top-level keys)
  [harness.codex.hooks]                --> .codex/hooks.json (never inline)
  [[subagents]] + [harness.codex.subagent.<slug>] --> .codex/agents/<slug>.toml
  [skills]                             --> .agents/skills/<name>/SKILL.md (default)
                                       --> .codex/skills/<name>/SKILL.md (with --codex-specific-skills)

import (harness --> theta):
  AGENTS.md (root walk, concatenated)  --> [instructions].system (opaque body)
  .codex/config.toml [mcp_servers.<n>] --> [tools] + [harness.codex.tool.<name>]
  .codex/config.toml (other top keys)  --> [harness.codex]
  .codex/config.toml [hooks]           --> [harness.codex.hooks]
  .codex/hooks.json                    --> [harness.codex.hooks]
  .codex/agents/<name>.toml            --> [[subagents]] + [harness.codex.subagent.<slug>]
  .agents/skills/<name>/SKILL.md       --> [skills] (canonical cross-agent)
  .codex/skills/<name>/SKILL.md        --> [skills] (legacy codex-specific path)

out-of-scope (NOT imported, NOT touched):
  ~/.codex/**                       (user-level)
  /etc/codex/**                     (system-level)
  requirements.toml                 (admin-enforced layer)
  .codex/rules/*.rules              (Starlark exec-policy, no portable equivalent)
  AGENTS.override.md                (override file, layer-specific)
  .codex-plugin/                    (plugin marketplace bundles)
  auth.json, history.jsonl          (credentials, transcripts)
";

const REFS: &str = "\
ref: https://developers.openai.com/codex/config-reference   (authoritative key list)
ref: https://developers.openai.com/codex/config-basic       (precedence, feature flags)
ref: https://developers.openai.com/codex/config-advanced    (profiles, hooks, env)
ref: https://developers.openai.com/codex/guides/agents-md   (AGENTS.md discovery, 32 KiB cap)
ref: https://developers.openai.com/codex/mcp                ([mcp_servers.<id>] schema)
ref: https://developers.openai.com/codex/hooks              (events, handler schema)
ref: https://developers.openai.com/codex/skills             (.agents/skills/ canonical)
ref: https://developers.openai.com/codex/subagents          (.codex/agents/<name>.toml as config layer)
ref: https://developers.openai.com/codex/rules              (.codex/rules/*.rules Starlark)
";

const IMPORT_OPAQUE_AGENTS_MD: &str = "\
  opaque AGENTS.md     AGENTS.md is treated as opaque system-prompt content.
                       no H1/heading parsing, no identity extraction from the
                       file body. agent identity comes from the project
                       directory. theta does NOT slugify the first heading.
                       ref: https://developers.openai.com/codex/guides/agents-md
";

const IMPORT_AGENTS_MD_WALK: &str = "\
  AGENTS.md walk       Codex concatenates AGENTS.md files from the project
                       root down to cwd, plus AGENTS.override.md per directory.
                       theta imports only the project-root AGENTS.md. nested
                       AGENTS.md and AGENTS.override.md are NOT merged on
                       import; the importer emits a hint per detected file.
                       cast always emits a single root AGENTS.md.
                       ref: https://developers.openai.com/codex/guides/agents-md
";

const IMPORT_MCP_TYPED_VS_EXTRAS: &str = "\
  mcp typed vs extras  .codex/config.toml [mcp_servers.<name>] is split: typed
                       keys (command, args, env, url, headers) go to [tools];
                       everything else goes to [harness.codex.tool.<name>] and
                       round-trips on cast. preserved extras include:
                       env_vars, cwd, bearer_token_env_var, env_http_headers,
                       oauth_resource, scopes, experimental_environment,
                       startup_timeout_sec, startup_timeout_ms, tool_timeout_sec,
                       enabled, required, enabled_tools, disabled_tools.
                       ref: https://developers.openai.com/codex/mcp
                       ref: https://developers.openai.com/codex/config-reference
";

const IMPORT_MCP_TRANSPORT_INFER: &str = "\
  mcp transport infer  codex has no explicit `type` enum on per-server
                       entries. theta infers transport from `command` (stdio)
                       vs `url` (streamable-http) at import time. round-trip
                       preserves the original field shape: command-only
                       servers stay command-only on cast.
                       ref: https://developers.openai.com/codex/mcp
";

const IMPORT_SUBAGENT_FULL_LAYER: &str = "\
  subagent as layer    .codex/agents/<name>.toml is a FULL config-layer per
                       the docs - it may carry any supported config.toml key.
                       theta carves out name, description, developer_instructions
                       (-->prompt), and model into typed [[subagents]] fields;
                       everything else round-trips through
                       [harness.codex.subagent.<slug>] verbatim. preserved
                       extras include: nickname_candidates, sandbox_mode,
                       model_reasoning_effort, model_reasoning_summary,
                       [mcp_servers.*] nested table, [permissions.*],
                       [skills.config], [features] overrides, etc.
                       ref: https://developers.openai.com/codex/subagents
";

const IMPORT_HOOKS_BOTH_FORMS: &str = "\
  hooks dual surface   codex accepts hooks in two surfaces: inline [hooks]
                       under .codex/config.toml, and .codex/hooks.json. theta
                       reads both into [harness.codex.hooks] without recording
                       which form the source used. cast always emits a single
                       .codex/hooks.json (see cast notes). source-form is
                       NOT preserved in the manifest.
                       ref: https://developers.openai.com/codex/hooks
";

const IMPORT_HOOKS_EVENTS_OPEN: &str = "\
  hook events open     the hooks docs page lists six events (SessionStart,
                       PreToolUse, PermissionRequest, PostToolUse,
                       UserPromptSubmit, Stop) but the openai/codex schema
                       folder also ships PreCompact and PostCompact (commit
                       527d52d). theta does NOT validate the event enum;
                       unknown events pass through verbatim.
                       ref: https://github.com/openai/codex/tree/main/codex-rs/hooks/schema/generated
";

const IMPORT_SKILLS_BOTH_PATHS: &str = "\
  skills dual location codex reads .agents/skills/ (canonical, cross-agent)
                       and .codex/skills/ (legacy, codex-only). theta imports
                       skills from both directories. if the same skill name
                       appears in both, the .agents/skills/ entry wins and the
                       .codex/skills/ duplicate is dropped with a hint.
                       ref: https://developers.openai.com/codex/skills
";

const IMPORT_RULES_NOT_IMPORTED: &str = "\
  .codex/rules/        .rules files are Starlark exec-policy, NOT instruction
                       rules. theta does NOT import them into
                       [instructions.rules]; a hint is emitted per detected
                       file. the Starlark source is left in place. there is
                       no portable equivalent.
                       ref: https://developers.openai.com/codex/rules
";

const IMPORT_PROFILES_OPAQUE: &str = "\
  [profiles.<name>]    codex named profiles are codex-specific. they are
                       preserved opaquely under [harness.codex.profiles] via
                       the typed config's extras passthrough, including any
                       nested [profiles.<name>.mcp_servers.*] etc.
                       ref: https://developers.openai.com/codex/config-advanced
";

const IMPORT_FEATURES_OPAQUE: &str = "\
  [features] table     codex feature flags (multi_agent, undo, shell_snapshot,
                       codex_hooks, fast_mode, memories, personality, apps,
                       prevent_idle_sleep, skill_mcp_dependency_install, ...)
                       round-trip verbatim through [harness.codex.features].
                       deprecated flags (web_search, web_search_cached,
                       web_search_request) pass through unchanged.
                       ref: https://developers.openai.com/codex/config-basic
";

const IMPORT_TOML_COMMENTS: &str = "\
  TOML comments        # comments in .codex/config.toml and subagent TOML
                       files are stripped during toml_edit re-serialization.
";

const IMPORT_JSONC_COMMENTS: &str = "\
  JSONC comments       // comments in .codex/hooks.json are stripped during
                       JSONC --> JSON parsing.
";

const IMPORT_CRLF: &str = "\
  CRLF normalization   Windows CRLF line endings normalize to LF on import.
";

const CAST_AGENTS_MD_FLAT: &str = "\
  rules flattened      cast writes a single root-level AGENTS.md containing
                       identity, system prompt, then every rule appended as
                       `## <rule-name>` H2 sections separated by `---`. codex
                       has no per-rule conditional activation, so this is the
                       only portable representation. import has no reverse
                       split: theta-origin manifest with N rules round-trips
                       through codex as system.md = whole AGENTS.md, zero
                       rules. by design.
                       ref: https://developers.openai.com/codex/guides/agents-md
";

const CAST_AGENTS_MD_SIZE: &str = "\
  AGENTS.md size cap   codex truncates AGENTS.md once the combined size
                       reaches project_doc_max_bytes (default 32 KiB). files
                       past the cap are silently dropped. cast emits a hint
                       when the output approaches the cap; theta does NOT
                       split or chunk the content automatically.
                       ref: https://developers.openai.com/codex/guides/agents-md
";

const CAST_HOOKS_JSON: &str = "\
  hooks output form    cast always writes .codex/hooks.json. inline [hooks]
                       in config.toml is never emitted, even when the original
                       source used inline form. both forms are semantically
                       equivalent to codex (it warns when both exist in the
                       same layer). this avoids the dual-source warning by
                       construction.
                       ref: https://developers.openai.com/codex/hooks
                       ref: https://developers.openai.com/codex/config-advanced
";

const CAST_SUBAGENT_KEBAB: &str = "\
  subagent filenames   cast uses kebab_case(name) for .codex/agents/<slug>.toml.
                       extras lookup tries the slug first, then the original
                       name, to tolerate manifests authored either way.
";

const CAST_SKILLS_DEFAULT: &str = "\
  skills default path  cast emits skills to .agents/skills/<name>/ by default
                       (canonical cross-agent location per current codex docs).
                       use --codex-specific-skills to emit to .codex/skills/
                       instead. round-trip is location-tolerant: equality
                       checkers treat the two paths as equivalent.
                       ref: https://developers.openai.com/codex/skills
";

const CAST_NO_RULES_DIR: &str = "\
  no .codex/rules/     cast does NOT emit .rules files. exec-policy is
                       codex-specific and authored by hand. theta carries
                       existing .codex/rules/ files only as opaque hints on
                       import; cast leaves the directory untouched.
";

const CAST_MCP_IN_CONFIG: &str = "\
  mcp lives in config  [tools] and [harness.codex.tool.<name>] cast to
                       [mcp_servers.<name>] inside .codex/config.toml. codex
                       has no separate .mcp.json at project scope (that path
                       exists only in plugin bundles, out of scope here).
                       ref: https://developers.openai.com/codex/mcp
";

const CAST_TOML_REORDER: &str = "\
  TOML key order       toml_edit re-serializes config.toml and subagent files;
                       top-level keys, [table] order, and inline-table style
                       are NOT guaranteed to match the source. semantic
                       content is preserved.
";

const CAST_JSON_REORDER: &str = "\
  JSON key order       .codex/hooks.json keys are serialized via
                       serde_json::to_string_pretty and may be reordered
                       relative to the source. event identity and handler
                       values are preserved.
";

const CAST_TRAILING_NEWLINE: &str = "\
  trailing newline     cast output always ends with \\n (POSIX). files that
                       originally lacked a trailing newline will differ.
";

const RT_TOML_QUOTE_STYLE: &str = "\
  TOML quote style     string values may switch between basic strings,
                       literal strings, and multi-line forms during
                       round-trip. semantically equivalent.
";

const RT_TOML_TABLE_STYLE: &str = "\
  TOML table style     inline tables vs `[table]` headers are not guaranteed
                       to match the source. theta uses block tables by default.
";

const RT_SKILL_LOCATION: &str = "\
  skill relocation     skills found under .codex/skills/ on import may emit
                       to .agents/skills/ on cast (the new canonical path),
                       and vice versa with --codex-specific-skills. equality
                       checkers tolerate the relocation; the SKILL.md content
                       and supporting files are preserved byte-for-byte.
                       ref: https://developers.openai.com/codex/skills
";

const RT_RULES_LOSSY: &str = "\
  theta rules --> codex theta-origin manifests with N rules in
                       [instructions.rules] cast to a single AGENTS.md.
                       reimporting that AGENTS.md yields zero rules and one
                       system prompt. WONTFIX: codex has no portable
                       per-rule conditional activation surface.
";

const NA_USER_LEVEL: &str = "\
  ~/.codex/**          user-level codex config (~/.codex/config.toml,
                       ~/.codex/agents/, ~/.codex/skills/, ~/.codex/rules/,
                       auth.json, history.jsonl) is per-user state and out
                       of scope for theta, which is project-scoped. cast
                       does not touch user-level paths; import does not read
                       them.
                       ref: https://developers.openai.com/codex/config-advanced
";

const NA_REQUIREMENTS_TOML: &str = "\
  requirements.toml    admin-enforced enterprise restriction layer. theta
                       does not import or cast requirements.toml. when
                       present, it constrains what codex accepts from project
                       config, but the file itself is not theta's surface.
                       ref: https://developers.openai.com/codex/config-reference
";

const NA_AGENTS_OVERRIDE: &str = "\
  AGENTS.override.md   per-layer override file. codex reads it before
                       AGENTS.md and uses ONLY the first non-empty file at
                       that layer. theta neither imports nor emits override
                       files; cast always writes AGENTS.md and leaves any
                       existing AGENTS.override.md in place.
                       ref: https://developers.openai.com/codex/guides/agents-md
";

const NA_PLUGINS: &str = "\
  .codex-plugin/       plugin manifests (.codex-plugin/plugin.json,
                       optional hooks/hooks.json, .mcp.json, .app.json) are
                       distribution bundles. theta does not import individual
                       plugin manifests; plugin-installed surfaces appear in
                       the runtime config layers and round-trip there.
                       ref: https://developers.openai.com/codex/hooks
";

const NA_EXEC_POLICY: &str = "\
  .codex/rules/        Starlark exec-policy files (.rules) gate which
                       commands can run outside the sandbox. they have no
                       portable theta equivalent and are not imported. the
                       importer emits a hint when these files are detected.
                       ref: https://developers.openai.com/codex/rules
";

/// Assemble import notes from individual constants.
pub(crate) fn import_notes() -> String {
    format!(
        "Codex CLI -- import notes (theta cast from codex-cli)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         import-specific:\n\
         {IMPORT_OPAQUE_AGENTS_MD}\
         {IMPORT_AGENTS_MD_WALK}\
         {IMPORT_MCP_TYPED_VS_EXTRAS}\
         {IMPORT_MCP_TRANSPORT_INFER}\
         {IMPORT_SUBAGENT_FULL_LAYER}\
         {IMPORT_HOOKS_BOTH_FORMS}\
         {IMPORT_HOOKS_EVENTS_OPEN}\
         {IMPORT_SKILLS_BOTH_PATHS}\
         {IMPORT_RULES_NOT_IMPORTED}\
         {IMPORT_PROFILES_OPAQUE}\
         {IMPORT_FEATURES_OPAQUE}\
         {IMPORT_TOML_COMMENTS}\
         {IMPORT_JSONC_COMMENTS}\
         {IMPORT_CRLF}\n\
         round-trip:\n\
         {RT_TOML_QUOTE_STYLE}\
         {RT_TOML_TABLE_STYLE}\
         {RT_SKILL_LOCATION}\
         {RT_RULES_LOSSY}\n\
         not applicable:\n\
         {NA_USER_LEVEL}\
         {NA_REQUIREMENTS_TOML}\
         {NA_AGENTS_OVERRIDE}\
         {NA_PLUGINS}\
         {NA_EXEC_POLICY}"
    )
}

/// Assemble cast notes from individual constants.
pub(crate) fn cast_notes() -> String {
    format!(
        "Codex CLI -- cast notes (theta cast to codex-cli)\n\n\
         {SURFACE_MAP}\n\
         {REFS}\n\
         cast-specific:\n\
         {CAST_AGENTS_MD_FLAT}\
         {CAST_AGENTS_MD_SIZE}\
         {CAST_HOOKS_JSON}\
         {CAST_SUBAGENT_KEBAB}\
         {CAST_SKILLS_DEFAULT}\
         {CAST_NO_RULES_DIR}\
         {CAST_MCP_IN_CONFIG}\
         {CAST_TOML_REORDER}\
         {CAST_JSON_REORDER}\
         {CAST_TRAILING_NEWLINE}\n\
         round-trip:\n\
         {RT_TOML_QUOTE_STYLE}\
         {RT_TOML_TABLE_STYLE}\
         {RT_SKILL_LOCATION}\
         {RT_RULES_LOSSY}\n\
         not applicable:\n\
         {NA_USER_LEVEL}\
         {NA_REQUIREMENTS_TOML}\
         {NA_AGENTS_OVERRIDE}\
         {NA_PLUGINS}\
         {NA_EXEC_POLICY}"
    )
}
