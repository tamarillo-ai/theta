---
name: aoa-workspace-recon
description: Map the AoA sibling workspace and ownership boundaries before cross-repo work. Use when a task touches multiple aoa-* repos, workspace-root Codex wiring, shared surfaces, or unclear ownership. Do not use for narrow single-repo edits when ownership is already obvious.
---

# AoA Workspace Recon

Your job is to orient first, not to rush.

## When to use
- Cross-repo work
- Workspace-root configuration
- Ownership uncertainty
- Any task that may touch more than one AoA repo

## Required tools
Use the AoA MCP servers named in `agents/openai.yaml`.

Start with:
1. `workspace_health`
2. `workspace_repo_map`
3. `workspace_surface_crosswalk`

If the task might touch harvest, lineage, or seeds, also inspect:
- `stats_catalog`
- `seed_route_catalog`

## Output contract
Return a short structured map with:

1. **Owning repo(s)**
2. **Non-owner supporting surfaces**
3. **Boundary warnings**
4. **Suggested first bounded change**
5. **Validation path**

## Boundary rules
- Do not treat `aoa_stats` as owner truth.
- Do not treat `Dionysus` as owner truth.
- Do not invent authority in `aoa-sdk` or `aoa-routing`.
- If the task is actually single-repo and obvious, say so and keep the route narrow.
