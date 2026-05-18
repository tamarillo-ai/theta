---
name: om-session-acceptance
description: Preflight, scaffold, and execute the OM session acceptance pipeline for this repository. Use when Codex needs to turn loose task materials into a normalized workspace, export a session bundle, run harness acceptance checks, or generate a minimal task workspace skeleton for incomplete session artifacts.
---

# OM Session Acceptance

Use this skill as the project-specific entrypoint for session acceptance work.

## Read Order

1. Read [`.codex/harness/evals/SESSION_ACCEPTANCE.md`](../../harness/evals/SESSION_ACCEPTANCE.md).
2. Read [`references/workflow-map.md`](references/workflow-map.md).
3. Read only the schema and eval spec files needed for the current task.

## Workflow

1. Start with a preflight scan:

   ```powershell
   python .codex/skills/om-session-acceptance/scripts/session_acceptance_entry.py `
     --source-root <path> `
     --mode preflight
   ```

2. Summarize:
   - detected source mode
   - discovered transcript, metadata, manifest, and artifacts
   - schema or structure failures
   - files that would be written
   - downstream commands that would run

3. Before any write, show the user the planned workspace or bundle path and explain why those files are needed.

4. After user approval, execute the pipeline:

   ```powershell
   python .codex/skills/om-session-acceptance/scripts/session_acceptance_entry.py `
     --source-root <path> `
     --mode execute `
     --validate
   ```

5. If inputs are incomplete, generate only the minimal workspace skeleton and stop:

   ```powershell
   python .codex/skills/om-session-acceptance/scripts/session_acceptance_entry.py `
     --source-root <path> `
     --mode execute `
     --scaffold-missing
   ```

6. Use `--dry-run` whenever the user wants to review the exact downstream actions first.

## Boundaries

- Reuse the existing harness scripts and schemas. Do not re-implement bundle, workspace, or acceptance contracts in prose.
- Treat `metadata.json` as valid output and generated placeholder artifacts as intentionally incomplete input. Do not present placeholder artifacts as completed evidence.
- If the wrapper generates a scaffold, stop after scaffold creation and tell the user which files still need real content.
- When the source is already a bundle, execute acceptance directly instead of rebuilding workspace or bundle artifacts.

