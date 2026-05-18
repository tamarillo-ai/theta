# AGENTS.md

## Guidance for `.codex/`

`.codex/` holds Codex-plane projection, regeneration, wrapper, and local operator-adjacent surfaces for the public entry repository. It may help agents orient in the workspace, but it does not own AoA doctrine, runtime truth, or sibling repo contracts.

Keep projected copies secondary. Source-owned files in this repository may be edited; installed live copies should be treated as evidence of projection drift, not authority.

No secrets, tokens, private hostnames, unreviewed hooks, or hidden local automation may be added here. Any mutating helper must stay bounded, documented, reversible, and route-aware.

Verify with:

```bash
python scripts/validate_nested_agents.py
python scripts/audit_agents_map.py --workspace-root <workspace-root> --write generated/agents_map.min.json --markdown docs/AGENTS_MAP.md
```
