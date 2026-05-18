---
name: aoa-seed-route-inspect
description: Inspect Dionysus seed staging, wave context, and planting rules for a reviewed candidate. Use when moving a reusable candidate toward Dionysus, checking staging state, or confirming planting constraints and trace.
---

# AoA Seed Route Inspect

Inspect the seed path without pretending the seed-garden is the final owner.

## Required read path
1. Start with `seed_route_catalog`.
2. If a specific seed is named, read `seed_registry_entry`.
3. If the seed is live or near-live, inspect `seed_next_live` and `seed_wave_context`.
4. If staging is involved, inspect `seed_staging_note`.
5. If planting decisions matter, read `seed_planting_rules`.

## Output contract
Return:

1. **Seed / candidate identity**
2. **Current lifecycle posture**
3. **Relevant wave or staging context**
4. **Planting constraints**
5. **What still belongs in Dionysus**
6. **What must be decided by the owner repo**

## Boundary rules
- Staging notes are weaker than owner-repo truth.
- Planting trace is lineage evidence, not queue authority.
- If this should not enter Dionysus, say so directly.
