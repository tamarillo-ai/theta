# FastCuda Agent Guide

This repository uses Codex's official project configuration surfaces:

- root `AGENTS.md` for durable project instructions
- `.codex/config.toml` for project-scoped Codex runtime config
- `.codex/rules/*.rules` for command rules
- `.codex/agents/*.md` for custom subagents
- `.codex/skills/*/SKILL.md` for project skills

Do not add repository agent configuration back into ad hoc manifests such as
`.codex/project.toml`. Keep Codex configuration in the official surfaces above.

## Mission

FastCuda is a handwritten CUDA operator workspace focused on:

- GEMM
- FlashAttention
- performance benchmarking
- performance analysis
- environment diagnosis

## Supported Targets

- build system: CMake
- host language standard: C++11
- CUDA language standard: C++11
- host platforms: Windows and Linux
- supported CUDA toolkits: 12.8.x and 13.0.x
- default CUDA architectures:
  - `89` for GeForce RTX 4090
  - `120` for GeForce RTX 5060
- default device tiers:
  - RTX 4090: 24 GB
  - RTX 5060: 8 GB

## Working Rules

- Prefer direct CUDA C++ and NVCC-compatible code.
- State GPU architecture assumptions explicitly.
- Keep baseline kernels separate from optimized kernels.
- Make tile sizes, launch geometry, memory strategy, and accumulation type explicit.
- Treat RTX 4090 and RTX 5060 as separate benchmark tiers.
- Do not make performance claims without benchmark or profiler artifacts.

## Standard Workflow

1. Inspect the target operator, shape range, dtype, and device tier.
2. Refresh environment state with `scripts/env/probe-env.ps1` when needed.
3. Implement or revise the kernel in a narrow change set.
4. Benchmark through `scripts/perf/run-benchmark.ps1`.
5. Profile with `scripts/perf/profile-ncu.ps1` or `scripts/perf/profile-nsys.ps1` only after a reproducible benchmark exists.
6. Record artifacts under `artifacts/`.

## Repository Layout

- Codex config:
  - `AGENTS.md`
  - `.codex/config.toml`
  - `.codex/rules/`
  - `.codex/agents/`
  - `.codex/skills/`
- project docs:
  - `docs/`
  - `docs/prompts/`
- build and execution:
  - `CMakeLists.txt`
  - `scripts/`
  - `benchmarks/`
  - `src/`
- machine-readable presets:
  - `configs/`

## Prompt Files

Prompt files live under `docs/prompts/`. They are reusable task briefs for
human or agent-driven work, but they are not official Codex configuration
surfaces.

## Hook And Tool Scripts

Hook and tool wrappers stay under `scripts/`. They are regular project assets,
not Codex configuration modules.

## Subagent Routing

- Use `kernel-architect` for tiling, dataflow, and launch planning.
- Use `kernel-optimizer` for data-backed kernel optimization changes.
- Use `perf-analyst` for benchmark and profiler interpretation.
- Use `env-investigator` for CUDA, driver, compiler, or profiler environment issues.
