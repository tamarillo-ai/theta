// Codex CLI caster
//
// cast (theta --> harness):
//   identity + system prompt + all rules --> AGENTS.md (concatenated, rules flattened)
//   [tools] + [harness.codex.tool.<name>] --> .codex/config.toml [mcp_servers.<name>]
//   [harness.codex]                      --> .codex/config.toml (top-level keys, opaque passthrough)
//   [harness.codex.hooks]                --> .codex/hooks.json (never inline)
//   [[subagents]] + [harness.codex.subagent.<slug>] --> .codex/agents/<slug>.toml
//   [skills]                             --> .agents/skills/<name>/SKILL.md (canonical default)
//                                        --> .codex/skills/<name>/SKILL.md (with codex_specific_skills)
//
// import (harness --> theta):
//   AGENTS.md (root, opaque)         --> [instructions].system (no heading parse)
//   .codex/config.toml [mcp_servers] --> [tools] + [harness.codex.tool.<name>]
//   .codex/config.toml (other keys)  --> [harness.codex] (lossless passthrough)
//   .codex/config.toml [hooks]       --> [harness.codex.hooks]
//   .codex/hooks.json                --> [harness.codex.hooks]
//   .codex/agents/<name>.toml        --> [[subagents]] + [harness.codex.subagent.<slug>]
//   .agents/skills/*/SKILL.md        --> [skills] (canonical cross-agent)
//   .codex/skills/*/SKILL.md         --> [skills] (legacy codex-specific path)
//
// out-of-scope (NOT imported, NOT touched):
//   ~/.codex/**             (user-level)
//   /etc/codex/**           (system-level)
//   requirements.toml       (admin-enforced layer)
//   .codex/rules/*.rules    (Starlark exec-policy; no portable equivalent)
//   AGENTS.override.md      (per-layer override)
//   .codex-plugin/          (plugin marketplace bundles)
//
// ref: https://developers.openai.com/codex/config-reference   (authoritative key list)
// ref: https://developers.openai.com/codex/config-basic       (precedence, feature flags)
// ref: https://developers.openai.com/codex/config-advanced    (profiles, hooks, env)
// ref: https://developers.openai.com/codex/guides/agents-md   (AGENTS.md discovery, 32 KiB cap)
// ref: https://developers.openai.com/codex/mcp                ([mcp_servers.<id>] schema)
// ref: https://developers.openai.com/codex/hooks              (events, handler schema)
// ref: https://developers.openai.com/codex/skills              (.agents/skills/ canonical)
// ref: https://developers.openai.com/codex/subagents          (.codex/agents/<name>.toml as config layer)
// ref: https://developers.openai.com/codex/rules              (.codex/rules/*.rules Starlark)

mod cast;
mod import;
pub(crate) mod notes;

pub struct CodexCli;

#[cfg(test)]
mod tests;
