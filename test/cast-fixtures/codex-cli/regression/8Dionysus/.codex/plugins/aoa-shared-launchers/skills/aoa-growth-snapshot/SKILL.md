---
name: aoa-growth-snapshot
description: Produce a bounded AoA growth snapshot across workspace orientation, derived stats, and seed-garden context. Use when preparing or reviewing closeout, harvest, candidate lineage, seed staging, planting, progression, or self-diagnose/self-repair follow-through.
---

# AoA Growth Snapshot

Build one bounded snapshot across the AoA federation without flattening the layers.

## Required read path
1. Use `workspace_surface_crosswalk` to orient the task.
2. Use `stats_catalog` first, then only the few stats surfaces needed.
3. If seed posture matters, inspect `seed_route_catalog` and the most relevant seed or wave surface.

## Output contract
Return these sections:

1. **Task posture**
2. **Owner and derived layers involved**
3. **Observed lineage / growth evidence**
4. **Seed or planting relevance**
5. **Risk of false promotion or misrouting**
6. **One next bounded move**

## Boundary rules
- `aoa_stats` is derived.
- `Dionysus` is staging and route context.
- If owner truth is missing, say what owner repo must be checked next.
- Prefer a compact snapshot over exhaustive dumping.
