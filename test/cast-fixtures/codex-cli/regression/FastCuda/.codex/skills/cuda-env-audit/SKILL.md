---
name: cuda-env-audit
description: Use when CUDA availability, compiler setup, or profiler visibility may be broken or inconsistent.
---

# CUDA Environment Audit

Use this skill when CUDA availability or toolchain correctness is uncertain.

## Workflow

1. Run `scripts/env/probe-env.ps1`.
2. Check for `nvidia-smi`, `nvcc`, `ncu`, and `nsys`.
3. Save environment output under `artifacts/env/`.
4. Report mismatches between driver, runtime, and toolkit visibility.

## Output Contract

- one environment snapshot path
- one concise diagnosis
- one next action
