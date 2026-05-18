# GPT/Codex WSL Sysop Repo

<!-- BEGIN SYSOP GATE DOCTRINE -->
## Sysop Gate Doctrine
- All material changes must flow through `Researcher -> Challenger -> Implementer -> Verifier`.
- The hard enforcement point for material changes is `sysop/sysop-gate.sh`.
- No material change may be treated as complete, and no commit may be created by the steady-state gate, unless all four stages complete successfully.
- Researcher and Challenger establish and challenge evidence first. Implementer acts only on a passed challenge. Verifier must confirm the final state before commit.
- Primary docs, official changelogs, and direct local reproduction outrank issue trackers, forums, and other advisory sources.
- If the gate returns `BLOCK` or `FAIL`, do not bypass it by direct editing unless the user explicitly waives the gate for that task.
- For steady-state material changes, this doctrine outranks any older manual workflow text below if they conflict.
<!-- END SYSOP GATE DOCTRINE -->


## Index-first
- Read `AGENTS.md` first, then `sysop/README_INDEX.md` before acting.

## Repo-local Codex operating scaffold
- The repo ships `.codex/config.toml` and `.codex/agents/*.toml` to define a recursive audit mode.
- This scaffold does not override this `AGENTS.md`; repo safety boundaries and plan/approval rules still win.
- For every material finding or change, autonomously run the chain `researcher -> challenger -> implementer -> verifier`.
- If thread caps or runtime limits prevent parallel spawning, run that chain sequentially and do not skip a role.
- Challenger and verifier are mandatory gates, not optional narration.
- No material change may be treated as done unless it survives adversarial challenge and local verification.
- Treat primary docs, official changelogs, and direct local reproduction as source-of-truth evidence. Treat issue trackers, forum posts, and niche discussions as advisory until corroborated.
- This is an instruction contract, not proof of hard runtime binding; treat enforcement as unproven unless runtime evidence demonstrates it.
- In local `codex-cli 0.111.0`, fresh `codex exec` controls showed that no-flag search can exist without a CLI `--search` flag, repo-local `web_search = "disabled"` suppresses search, and `codex exec -c 'web_search="live"'` restores it. Treat no-flag search success as insufficient to prove repo-local live mode by itself.
- If the current session started before the repo-local `.codex/` files were loaded, restart from the repo root/worktree to activate them; do not claim the mode changed mid-session.
- In the dedicated recursive audit worktree/mode, native Codex web search is allowed for contemporary upstream research tied to audit findings.
- Native Codex web search does not authorize outbound shell networking; keep shell command writes and network posture repo-local and constrained.

## Codex operator contract (non-negotiable)
- Start in PLAN MODE: no edits and no write-producing commands until a human approves the plan.
- Intake first: identify tasks, then choose sequential vs parallel; if parallel, one worktree per task (`wt/<task-slug>`).
- One worktree per task: never mix unrelated changes; each worktree gets its own branch.
- Treat a worktree as an operational boundary, not a guaranteed runtime sandbox; verify cwd/branch/worktree after `resume`, `fork`, or `apply`.
- Local runtime control showed that explicit `codex exec resume <SESSION_ID>` can rebind to the caller worktree. Treat `fork` as safer only in the narrow sense that it currently surfaces the cwd decision explicitly.
- Verification required: every change ends with the relevant tests/build/lint (or closest available verification) and reported results.
- Falsifiable debugging: for each bug, propose 2–3 hypotheses, then collect the minimum evidence to confirm one.
- Auditable work: small commits, stable reproduction steps, and clear per-worktree notes.
- Durable repo memory: after success, append an “avoid repeating this” note to `NOTES.md` (mistake → fix → proof command).

## Workflow (A→G)

### A) Intake → choose concurrency shape
- Identify tasks (feature A, bug B, refactor C).
- Decide: sequential vs parallel.
- If parallel: create one worktree per task (repo boundary, not a guaranteed runtime sandbox) before editing anything.

### B) Plan mode (no edits yet)
- Define “done means…” checks (tests passing, build clean, UI fixed).
- For each bug: generate 2–3 plausible hypotheses (especially UI).
- Decide minimal evidence to confirm one hypothesis.
- Draft an execution checklist (ordered steps + verification gates).
- STOP and ask for approval before proceeding to execution.

### C) State isolation (worktrees)
- Create one worktree per task: `wt/<task-slug>`.
- Ensure each worktree has its own branch.
- Helper: `./sysop/wt-new.sh <task-slug> [base-ref]`.
- Keep cross-task changes forbidden unless explicitly planned.
- Before editing after `resume`, `fork`, `apply`, or session restore, verify `pwd`, `git branch --show-current`, and the intended worktree path.
- For fresh-session controls, do not treat `git worktree add ... HEAD` as faithful if the active `.codex/` scaffold is uncommitted; commit it first or mirror the active runtime scaffold into the disposable control explicitly.

### D) Execute (edits happen now)
- Apply changes exactly as per the approved plan.
- Keep commits small inside each worktree.
- For UI: collect evidence via browser automation if available; save artifacts to stable paths for diffing.

### E) Verification loop (non-negotiable)
- Run unit/integration tests.
- Run build/lint/typecheck.
- If UI: run Playwright (or equivalent) and compare screenshots.
- If failed: update hypothesis → gather only missing evidence → retry.

### F) Review + integrate
- Produce a concise review note per worktree: what changed, why it should work, how it was verified.
- Human reviews and commits/merges (or explicitly tells Codex to proceed).
- Optional: PR stacking/reorder when multiple PRs exist; keep each PR a composable slice.

### G) Durable memory capture (Codex-side)
- Write “gotchas” into `NOTES.md` (or `AGENTS.md`): mistake → fix → test that proves it.
- Record repeatable commands (copy/paste runnable).

## Safety boundaries (repo)
- Never run destructive ops: `rm -rf`, `git reset --hard`, `git clean -fdx`.
- Never write to `/etc`.
- Do not write outside this repo unless the user explicitly authorizes host-level remediation and you create a timestamped backup plus a rollback path first.
- Do not change WSL interop settings or mount options.
- Work local-first. Exception: in the dedicated recursive audit mode, native Codex web search may be used for contemporary upstream research.
- Do not use shell commands to fetch from the internet unless repo rules explicitly change; keep command-side network access disabled.
- Prefer Linux-native repos under `/home` (avoid `/mnt/c` unless required).

## Interop status (WSL ↔ Windows)
- As of `2026-01-15`, WSL is configured with `/etc/wsl.conf` `[interop] appendWindowsPath=true`, enabling calls to Windows binaries from WSL (for example `powershell.exe`, `pwsh.exe`, `powercfg.exe`).
- Treat Windows `.exe` calls as host actions: prefer a drive-backed cwd (`/mnt/c`) and beware PATH precedence (Windows tools can shadow Linux ones).
- Note: some sandboxed runners block WSL↔Windows interop even when PATH is present (symptom: `UtilBindVsockAnyPort: ... socket failed 1`); fall back to a normal interactive WSL shell or native Windows PowerShell.

## Safety boundaries (auto-fix mode)
Auto-applied (Level 1):
- ✅ Repo-scoped retries and artifact regeneration with backups + rollback commands.

Generated only (Level 2–4):
- ✅ PowerShell/scripts/instructions for manual review under `sysop/out/fixes/`.

Never auto-applied:
- ❌ Any host/system change (e.g., `.wslconfig`, power plan changes, `wsl --shutdown`, package installs).

## Operating rules
- Read-only first: propose changes before edits/installs.
- Evidence discipline: back non-obvious claims with `man`/`--help` output or command results.
- Idempotent edits only; avoid duplicate lines in dotfiles.
- Backups + rollback: before editing a file outside the repo, create `*.bak-YYYYMMDD-HHMMSS` next to it and print the rollback command.

## Output format (every response)
1) Plan (or Execution report if already approved)
2) Exact commands run + results
3) Files changed summary
4) Remaining risks / what would still break
5) Repo memory note to append (`AGENTS.md`/`NOTES.md`)

## Fresh-state practice
- If context gets messy, write a short summary to `learn/LEDGER.md` and continue.

## Repo layout
- `sysop/`: operator scripts (`preflight.sh`, `healthcheck.sh`) and local operator README.
- `sysop-report/`: living report(s); append updates rather than creating new reports unless necessary.
