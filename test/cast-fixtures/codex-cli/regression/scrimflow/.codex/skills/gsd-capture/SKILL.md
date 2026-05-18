---
name: "gsd-capture"
description: "Capture ideas, tasks, notes, and seeds to their destination"
metadata:
  short-description: "Capture ideas, tasks, notes, and seeds to their destination"
---

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-capture`.
- Treat all user text after `$gsd-capture` as `{{GSD_ARGS}}`.
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
Capture ideas, tasks, notes, and seeds to their appropriate destination in the GSD system.

Mode routing:
- **default** (no flag): Capture as a structured todo for later work → add-todo workflow
- **--note**: Zero-friction idea capture (append/list/promote) → note workflow
- **--backlog**: Add an idea to the backlog parking lot (999.x numbering) → add-backlog workflow
- **--seed**: Capture a forward-looking idea with trigger conditions → plant-seed workflow
- **--list**: List pending todos and select one to work on → check-todos workflow
</objective>

<routing>

| Flag | Destination | Workflow |
|------|-------------|----------|
| (none) | Structured todo in .planning/todos/ | add-todo |
| --note | Timestamped note file, list, or promote | note |
| --backlog | ROADMAP.md backlog section (999.x) | add-backlog |
| --seed | .planning/seeds/SEED-NNN-slug.md | plant-seed |
| --list | Interactive todo browser + action router | check-todos |

</routing>

<execution_context>
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/add-todo.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/note.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/add-backlog.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/plant-seed.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/check-todos.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/ui-brand.md
</execution_context>

<context>
Arguments: {{GSD_ARGS}}

Parse the first token of {{GSD_ARGS}}:
- If it is `--note`: strip the flag, pass remainder to note workflow
- If it is `--backlog`: strip the flag, pass remainder to add-backlog workflow
- If it is `--seed`: strip the flag, pass remainder to plant-seed workflow
- If it is `--list`: pass remainder (optional area filter) to check-todos workflow
- Otherwise: pass all of {{GSD_ARGS}} to add-todo workflow
</context>

<process>
1. Parse the leading flag (if any) from {{GSD_ARGS}}.
2. Load and execute the appropriate workflow end-to-end based on the routing table above.
3. Preserve all workflow gates from the target workflow (directory structure, duplicate detection, commits, etc.).
</process>
