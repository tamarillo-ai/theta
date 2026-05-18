# Changelog — `sysop-kernel`

## 1.0.0 — 2026-01-04

Initial implementation and Codex CLI `>=0.76.0` alignment:

- Added YAML frontmatter fields (`triggers`, `version`, `codex-version`) to enable progressive disclosure triggering.
- Preserved the progressive disclosure structure (`SKILL.md` + `references/`) and operator workflow (`./sysop/run.sh all`).
- Documented and cross-referenced the learning ledger integration (`learn/LEDGER.md`) and machine-readable rules (`learn/RULES.md`).
- Updated operator index docs to include contemporary Codex debugging patterns (notably the `/ps` command) and common WSL/Windows error recovery.
- Standardized on `approval_policy="on-request"` and offline-first operation (no MCP, no internet fetches) as the recommended operator posture.
