---
name: benchmark-harness
description: Use when creating or revising CUDA benchmark runners and result artifacts.
---

# Benchmark Harness

Use this skill when creating or updating benchmark runners.

## Workflow

1. Define benchmark shapes, dtypes, and device tier.
2. Add warmup and timed iterations.
3. Ensure outputs go to `artifacts/benchmarks/`.
4. Include an environment snapshot reference when the comparison matters.
5. Preserve machine-readable output.

## Minimum Metrics

- median latency
- p95 latency
- throughput when meaningful
