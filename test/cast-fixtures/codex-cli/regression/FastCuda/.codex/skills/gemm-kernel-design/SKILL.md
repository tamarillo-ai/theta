---
name: gemm-kernel-design
description: Use when designing or reviewing handwritten CUDA GEMM kernels.
---

# GEMM Kernel Design

Use this skill when building or reviewing GEMM kernels.

## Checklist

- matrix layout assumptions
- tile sizes
- threadblock and warp mapping
- accumulation precision
- edge handling

## Focus

- baseline kernel before advanced optimization
- explicit shared memory usage
- measurable benchmark target
