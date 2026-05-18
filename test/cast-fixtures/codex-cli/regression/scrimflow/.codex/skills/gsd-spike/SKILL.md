---
name: "gsd-spike"
description: "Spike an idea through experiential exploration, or propose what to spike next (frontier mode)"
metadata:
  short-description: "Spike an idea through experiential exploration, or propose what to spike next (frontier mode)"
---

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-spike`.
- Treat all user text after `$gsd-spike` as `{{GSD_ARGS}}`.
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
Spike an idea through experiential exploration — build focused experiments to feel the pieces
of a future app, validate feasibility, and produce verified knowledge for the real build.
Spikes live in `.planning/spikes/` and integrate with GSD commit patterns, state tracking,
and handoff workflows.

Two modes:
- **Idea mode** (default) — describe an idea to spike
- **Frontier mode** (no argument or "frontier") — analyzes existing spike landscape and proposes integration and frontier spikes

Does not require `$gsd-new-project` — auto-creates `.planning/spikes/` if needed.
</objective>

<execution_context>
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/spike.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/spike-wrap-up.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/ui-brand.md
</execution_context>

<runtime_note>
**Copilot (VS Code):** Use `vscode_askquestions` wherever this workflow calls `AskUserQuestion`.
</runtime_note>

<context>
Idea: {{GSD_ARGS}}

**Available flags:**
- `--quick` — Skip decomposition/alignment, jump straight to building. Use when you already know what to spike.
- `--text` — Use plain-text numbered lists instead of AskUserQuestion (for non-the agent runtimes).
- `--wrap-up` — Package spike findings into a persistent project skill for future build conversations. Runs the spike-wrap-up workflow.
</context>

<process>
Parse the first token of {{GSD_ARGS}}:
- If it is `--wrap-up`: strip the flag, execute the spike-wrap-up workflow from @C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/spike-wrap-up.md.
- Otherwise: pass all of {{GSD_ARGS}} as the idea to the spike workflow from @C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/spike.md end-to-end.

Preserve all workflow gates (prior spike check, decomposition, research, risk ordering, observability assessment, verification, MANIFEST updates, commit patterns).
</process>
