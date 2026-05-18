---
name: "gsd-fast"
description: "Execute a trivial task inline — no subagents, no planning overhead"
metadata:
  short-description: "Execute a trivial task inline — no subagents, no planning overhead"
---

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-fast`.
- Treat all user text after `$gsd-fast` as `{{GSD_ARGS}}`.
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
- `Task(model="...")` → omit (Codex uses per-role config, not inline model selection)
- `fork_context: false` by default — GSD agents load their own context via `<files_to_read>` blocks

Parallel fan-out:
- Spawn multiple agents → collect agent IDs → `wait(ids)` for all to complete

Result parsing:
- Look for structured markers in agent output: `CHECKPOINT`, `PLAN COMPLETE`, `SUMMARY`, etc.
- `close_agent(id)` after collecting results from each agent
</codex_skill_adapter>

<objective>
Execute a trivial task directly in the current context without spawning subagents
or generating PLAN.md files. For tasks too small to justify planning overhead:
typo fixes, config changes, small refactors, forgotten commits, simple additions.

This is NOT a replacement for $gsd-quick — use $gsd-quick for anything that
needs research, multi-step planning, or verification. $gsd-fast is for tasks
you could describe in one sentence and execute in under 2 minutes.
</objective>

<execution_context>
@/Users/diegocaminor/Documents/proyectos/anyone-ai/final-project/anyone-ai-interviewer-frontend/.codex/get-shit-done/workflows/fast.md
</execution_context>

<process>
Execute the fast workflow from @/Users/diegocaminor/Documents/proyectos/anyone-ai/final-project/anyone-ai-interviewer-frontend/.codex/get-shit-done/workflows/fast.md end-to-end.
</process>
