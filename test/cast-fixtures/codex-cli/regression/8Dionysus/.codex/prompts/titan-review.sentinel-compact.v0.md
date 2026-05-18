# Sentinel Compact Review v0

Spawn custom agent named `Sentinel`.

Do not spawn generic agent `reviewer`.

Input must include literal diff excerpts.

Task:
Review one narrow slice only. Return `PASS`, `BLOCK`, or `PARTIAL`.

Required output:
- verdict
- cited evidence refs
- blocking findings, if any
- stale/uncertain findings, if any

Must not:
- expand scope
- invent unseen files
- claim final closure without evidence
