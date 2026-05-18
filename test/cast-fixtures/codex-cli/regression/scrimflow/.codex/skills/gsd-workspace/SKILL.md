---
name: "gsd-workspace"
description: "Manage GSD workspaces — create, list, or remove isolated workspace environments"
metadata:
  short-description: "Manage GSD workspaces — create, list, or remove isolated workspace environments"
---

<codex_skill_adapter>
## A. Skill Invocation
- This skill is invoked by mentioning `$gsd-workspace`.
- Treat all user text after `$gsd-workspace` as `{{GSD_ARGS}}`.
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
Manage GSD workspaces with a single consolidated command.

Mode routing:
- **--new**: Create an isolated workspace with repo copies and independent .planning/ → new-workspace workflow
- **--list**: List active GSD workspaces and their status → list-workspaces workflow
- **--remove**: Remove a GSD workspace and clean up worktrees → remove-workspace workflow
</objective>

<routing>

| Flag | Action | Workflow |
|------|--------|----------|
| --new | Create workspace with worktree/clone strategy | new-workspace |
| --list | Scan ~$gsd-workspaces/, show summary table | list-workspaces |
| --remove | Confirm and remove workspace directory | remove-workspace |

</routing>

<execution_context>
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/new-workspace.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/list-workspaces.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/workflows/remove-workspace.md
@C:/Users/simon/Documents/GitHub/scrimflow/.codex/get-shit-done/references/ui-brand.md
</execution_context>

<context>
Arguments: {{GSD_ARGS}}

Parse the first token of {{GSD_ARGS}}:
- If it is `--new`: strip the flag, pass remainder (--name, --repos, --path, --strategy, --branch, --auto flags) to new-workspace workflow
- If it is `--list`: execute list-workspaces workflow (no argument needed)
- If it is `--remove`: strip the flag, pass remainder (workspace-name) to remove-workspace workflow
- Otherwise (no flag): show usage — one of --new, --list, or --remove is required
</context>

<process>
1. Parse the leading flag from {{GSD_ARGS}}.
2. Load and execute the appropriate workflow end-to-end based on the routing table above.
3. Preserve all workflow gates from the target workflow (validation, approvals, commits, routing).
</process>
