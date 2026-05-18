---
name: aoa-config-doctor
description: Diagnose AoA Codex wiring when shared launcher skills or AoA MCP servers are missing, misnamed, or disconnected. Use when /mcp is incomplete, dependencies do not resolve, or a skill should work but does not.
---

# AoA Config Doctor

Diagnose the wiring instead of guessing.

## First checks
1. Confirm the user is inside the intended AoA workspace.
2. Ask Codex to show `/mcp` and `/skills`.
3. Check that the expected MCP names are present:
   - `aoa_workspace`
   - `aoa_stats`
   - `dionysus`

## If local shell access is appropriate
Recommend the smallest real validators that already exist in the workspace:

```bash
WORKSPACE_ROOT="$(
python - <<'PY'
from pathlib import Path

start = Path.cwd().resolve()
for candidate in [start, *start.parents]:
    if not (candidate / "AOA_WORKSPACE_ROOT").exists():
        continue
    if (
        (candidate / "aoa-sdk").exists()
        and (candidate / "aoa-skills").exists()
        and (candidate / ".codex").exists()
    ):
        print(candidate)
        break
else:
    raise SystemExit("Could not find an AoA workspace root above the current directory.")
PY
)"

python "$WORKSPACE_ROOT/.codex/scripts/aoa_codex_doctor.py" \
  --workspace-root "$WORKSPACE_ROOT" \
  --write-report

python "$WORKSPACE_ROOT/aoa-skills/scripts/validate_skill_mcp_wiring.py" \
  --workspace-config "$WORKSPACE_ROOT/.codex/config.toml" \
  --paths "$WORKSPACE_ROOT"/.codex/plugins/aoa-shared-launchers/skills/*/agents/openai.yaml \
  --format text
```

Use the first command for workspace-root Codex seam drift.
Use the second command when the plugin is visible but MCP dependency names or skill metadata still look wrong.

## Output contract
Return:

1. **What is missing**
2. **Whether the problem is skill discovery or MCP wiring**
3. **Exact path to fix first**
4. **Shortest verification step**

## Boundary rules
- Prefer naming mismatches and path mismatches over vague blame.
- Do not recommend duplicating shared skills into every repo unless there is a clear reason.
