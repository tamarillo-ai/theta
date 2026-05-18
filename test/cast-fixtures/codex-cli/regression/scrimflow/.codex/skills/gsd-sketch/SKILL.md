---
name: "gsd-sketch"
description: "Sketch UI/design ideas with throwaway HTML mockups, or propose what to sketch next (frontier mode)"
metadata:
  short-description: "Sketch UI/design ideas with throwaway HTML mockups, or propose what to sketch next (frontier mode)"
---

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-sketch`.
- Treat all user text after `$gsd-sketch` as `{{GSD_ARGS}}`.
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
Explore design directions through throwaway HTML mockups before committing to implementation.
Each sketch produces 2-3 variants for comparison. Sketches live in `.planning/sketches/` and
integrate with GSD commit patterns, state tracking, and handoff workflows. Loads spike
findings to ground mockups in real data shapes and validated interaction patterns.

Two modes:
- **Idea mode** (default) — describe a design idea to sketch
- **Frontier mode** (no argument or "frontier") — analyzes existing sketch landscape and proposes consistency and frontier sketches

Does not require `$gsd-new-project` — auto-creates `.planning/sketches/` if needed.
</objective>

<execution_context>
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/sketch.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/sketch-wrap-up.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/ui-brand.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/sketch-theme-system.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/sketch-interactivity.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/sketch-tooling.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/sketch-variant-patterns.md
</execution_context>

<runtime_note>
**Copilot (VS Code):** Use `vscode_askquestions` wherever this workflow calls `AskUserQuestion`.
</runtime_note>

<context>
Design idea: {{GSD_ARGS}}

**Available flags:**
- `--quick` — Skip mood/direction intake, jump straight to decomposition and building. Use when the design direction is already clear.
- `--wrap-up` — Package sketch design findings into a persistent project skill for future build conversations. Runs the sketch-wrap-up workflow.
</context>

<process>
Parse the first token of {{GSD_ARGS}}:
- If it is `--wrap-up`: strip the flag, execute the sketch-wrap-up workflow from @C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/sketch-wrap-up.md end-to-end.
- Otherwise: execute the sketch workflow from @C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/sketch.md end-to-end.

Preserve all workflow gates (intake, decomposition, target stack research, variant evaluation, MANIFEST updates, commit patterns).
</process>
