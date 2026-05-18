---
name: ncu-profiling
description: Use for kernel-level analysis with Nsight Compute after a benchmark is reproducible.
---

# Nsight Compute Profiling

Use this skill for kernel-level analysis with Nsight Compute.

## Workflow

1. Confirm benchmark reproducibility first.
2. Run `scripts/perf/profile-ncu.ps1`.
3. Capture the output path in `artifacts/profiles/`.
4. Interpret occupancy, memory throughput, shared memory pressure, and instruction mix.

## Guardrail

Do not make optimization claims from one metric alone.
