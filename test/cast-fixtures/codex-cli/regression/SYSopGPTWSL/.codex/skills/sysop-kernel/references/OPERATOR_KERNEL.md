# Operator Kernel + Learning Loop (SYSopGPTWSL)

## Contract
- Index-first: read `AGENTS.md` and `sysop/README_INDEX.md` before acting.
- One entrypoint: `./sysop/run.sh all`
- Outputs (diffable, overwritten each run): `sysop/out/`
- Learning: append a small entry to `learn/LEDGER.md` after successful runs.

## Plan-first (Codex)
- Treat `./sysop/run.sh ...` as EXECUTION MODE because it writes under `sysop/out/` and appends to `learn/LEDGER.md`.
- In PLAN MODE, prefer read-only probes (`./sysop/preflight.sh`, `./sysop/healthcheck.sh`) to gather evidence.

## What `./sysop/run.sh all` does
1) `health`:
   - Writes `sysop/out/wsl_snapshot.txt`
   - Runs `sysop/preflight.sh`, `sysop/healthcheck.sh`, `sysop/drift-check.sh`
2) `bench`:
   - Writes `sysop/out/bench.txt`
   - Includes a tiny `/tmp` vs `/mnt/c` FS comparison
3) `snapshot`:
   - Writes `sysop/out/windows_snapshot.json` + `sysop/out/windows_snapshot.txt`
   - Runs Windows PowerShell from `/mnt/c` (drive-backed cwd) to avoid UNC cwd issues
4) `report`:
   - Writes `sysop/out/report.md` (currently generated via `sysop/perf/summarize.sh`)
5) Learning:
   - Appends an entry to `learn/LEDGER.md`

## Known WSL/Windows pitfalls (baked into the kernel)
- UNC cwd: Windows commands run from `/mnt/c` even when scripts live under `/home/...`.
- UTF-8 BOM: Windows PowerShell can emit a BOM in UTF-8 JSON; Linux readers should use `utf-8-sig`.
- `/mnt/c` perf: drvfs/9p is slower than Linux FS; keep repo under `/home`.
- `.wslconfig` caps apply only after `wsl --shutdown` (Windows PowerShell).

## Output format (operator)
- Use the repo output format in `AGENTS.md` (“Output format (every response)”).
