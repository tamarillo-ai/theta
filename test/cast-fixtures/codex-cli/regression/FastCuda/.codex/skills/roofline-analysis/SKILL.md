---
name: roofline-analysis
description: Use when deciding whether a CUDA kernel is likely memory-bound or compute-bound.
---

# Roofline Analysis

Use this skill when deciding whether a kernel is likely memory-bound or
compute-bound.

## Workflow

1. Collect shape, dtype, and measured runtime.
2. Estimate bytes moved and operations performed.
3. Compare arithmetic intensity against hardware limits.
4. Use the result to narrow optimization direction.

## Output Contract

- arithmetic intensity estimate
- likely bound type
- next optimization target
