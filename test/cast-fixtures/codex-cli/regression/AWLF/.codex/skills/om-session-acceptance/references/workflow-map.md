# OM Session Acceptance Workflow Map

## Source Of Truth

1. [AGENTS.md](../../../AGENTS.md)
   Defines the constitutional workflow rules and approval constraints.
2. [`.codex/AGENT_RACI.md`](../../../.codex/AGENT_RACI.md)
   Defines stage responsibilities and hand-off expectations.
3. [`.codex/harness/evals/SESSION_ACCEPTANCE.md`](../../../.codex/harness/evals/SESSION_ACCEPTANCE.md)
   Defines workspace, bundle, and acceptance usage for this harness.

## Runtime Files

- [`.codex/harness/scripts/materialize_task_workspace.py`](../../../.codex/harness/scripts/materialize_task_workspace.py)
  Normalizes loose materials into a task workspace.
- [`.codex/harness/scripts/export_session_bundle.py`](../../../.codex/harness/scripts/export_session_bundle.py)
  Exports a normalized workspace into a session bundle.
- [`.codex/harness/scripts/run_session_acceptance.py`](../../../.codex/harness/scripts/run_session_acceptance.py)
  Runs eval specs against a bundle.
- [`.codex/harness/scripts/harness_utils.py`](../../../.codex/harness/scripts/harness_utils.py)
  Holds shared roots, schema bindings, and stage inference helpers.

## Skill Entry Contract

- Always run the wrapper in `preflight` mode first unless the user explicitly asks for direct execution and the inputs are already known to be complete.
- The wrapper may write only to:
  - `workspaces/<name>/`
  - `runs/<name>/`
- If the input set is incomplete, the wrapper may generate:
  - `metadata.json`
  - `transcript.md`
  - `artifacts/*.json` placeholders
- Placeholder artifacts are intentionally invalid against schema. They mark work that still needs human completion.

## Lifecycle

1. `materials/` or another source directory
2. `workspaces/<name>/`
3. `runs/<name>/`
4. acceptance result in terminal output

Do not create an alternate lifecycle outside these directories.

