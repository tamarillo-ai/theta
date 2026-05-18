---
name: "gsd-ai-integration-phase"
description: "Generate an AI-SPEC.md design contract for phases that involve building AI systems."
metadata:
  short-description: "Generate an AI-SPEC.md design contract for phases that involve building AI systems."
---

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-ai-integration-phase`.
- Treat all user text after `$gsd-ai-integration-phase` as `{{GSD_ARGS}}`.
- If no arguments are present, treat `{{GSD_ARGS}}` as empty.

## B. AskUserQuestion → request_user_input Mapping
GSD workflows use `AskUserQuestion` (Claude Code syntax). Translate to Codex `request_user_input`:

Parameter mapping:
- `header` → `header`
- `question` → `question`
- Options formatted as `"Label" — description` → `{label: "Label", description: "description"}`
- Generate `id` from header: lowercase, replace spaces with underscores

Batched calls:
- `AskUserQuestion([q1, q2])` → single `request_user_input` with multiple entries in `questions[]`

Multi-select workaround:
- Codex has no `multiSelect`. Use sequential single-selects, or present a numbered freeform list asking the user to enter comma-separated numbers.

Execute mode fallback:
- When `request_user_input` is rejected (Execute mode), present a plain-text numbered list and pick a reasonable default.

## C. Task() → spawn_agent Mapping
GSD workflows use `Task(...)` (Claude Code syntax). Translate to Codex collaboration tools:

Direct mapping:
- `Task(subagent_type="X", prompt="Y")` → `spawn_agent(agent_type="X", message="Y")`
- `Task(model="...")` → omit. `spawn_agent` has no inline `model` parameter;
  GSD embeds the resolved per-agent model directly into each agent's `.toml`
  at install time so `model_overrides` from `.planning/config.json` and
  `~/.gsd/defaults.json` are honored automatically by Codex's agent router.
- `fork_context: false` by default — GSD agents load their own context via `<files_to_read>` blocks

Spawn restriction:
- Codex restricts `spawn_agent` to cases where the user has explicitly
  requested sub-agents. When automatic spawning is not permitted, do the
  work inline in the current agent rather than attempting to force a spawn.

Parallel fan-out:
- Spawn multiple agents → collect agent IDs → `wait(ids)` for all to complete

Result parsing:
- Look for structured markers in agent output: `CHECKPOINT`, `PLAN COMPLETE`, `SUMMARY`, etc.
- `close_agent(id)` after collecting results from each agent
</codex_skill_adapter>

<objective>
Create an AI design contract (AI-SPEC.md) for a phase involving AI system development.
Orchestrates gsd-framework-selector → gsd-ai-researcher → gsd-domain-researcher → gsd-eval-planner.
Flow: Select Framework → Research Docs → Research Domain → Design Eval Strategy → Done
</objective>

<execution_context>
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/ai-integration-phase.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/ai-frameworks.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/ai-evals.md
</execution_context>

<context>
Phase number: {{GSD_ARGS}} — optional, auto-detects next unplanned phase if omitted.
</context>

<process>
Execute @C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/ai-integration-phase.md end-to-end.
Preserve all workflow gates.
</process>
