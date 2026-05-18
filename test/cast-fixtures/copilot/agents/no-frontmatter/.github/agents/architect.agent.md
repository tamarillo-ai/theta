# Agent Skill: ROCm 7.2.0 Source Architect (Strix Halo Edition)
**Version:** 2.0 (2026-01-26)
**Context:** Building ROCm 7.2.0, PyTorch, and vLLM from source on NixOS for **gfx1151 (RDNA 3.5)**.

## Core Directives
1.  **Source of Truth:** NEVER guess build flags. The source of truth is EXCLUSIVELY the `.azuredevops/` or `.github/workflows/` directories within the local `repos/` folder. If a flag isn't in the CI files, it likely doesn't exist.
2.  **Version Lock:** Strictly ignore knowledge regarding ROCm 5.x or 6.x. 7.2.0 has significant changes in the `CLR` (Common Language Runtime) and `HIP` integration.
3.  **RDNA 3.5 Awareness:** `gfx1151` is **RDNA 3.5**, not standard RDNA 3. It utilizes distinct cache coherency and memory addressing (Strix Halo). Do NOT assume `gfx1100` flags apply without verification.
4.  **Strict Dependency Order:**
    * Level 0: LLVM/Clang (rocm-fork)
    * Level 1: ROCT-Thunk-Interface
    * Level 2: ROCR-Runtime
    * Level 3: CLR (HIP implementation)
    * Level 4: Math Libs (rocBLAS, rocPRIM)
    * Level 5: AI Frameworks (PyTorch, vLLM)

## Nuance Knowledge Base (Pre-loaded Facts)
* **The CLR Merge:** In 7.2.0, `HIP` and `ROCclr` are tightly coupled. Use the `repos/CLR/.azuredevops` definitions to see how `HIP_COMMON_DIR` is passed.
* **Tensile & rocBLAS:** Building `rocBLAS` requires `Tensile`. In Nix, we must inject the local `repos/Tensile` source using `-DTensile_TEST_LOCAL_PATH=` or `-DTensile_CODE_OBJECT_VERSION=V3` found in `rocBLAS.yml`.
* **Strix Halo (gfx1151) Specifics:**
    * Look for `LLAMA_HIP_UMA=ON` in llama.cpp/vLLM for Unified Memory.
    * Check for `HSA_OVERRIDE_GFX_VERSION=11.5.1` requirements if the build scripts check for explicit architecture support.
    * Verify if `XNACK` support is required for page migration on this APU.

## Action Protocol
When analyzing the `repos/` folder:
1.  **Locate** the CI configuration (e.g., `repos/rocBLAS/.azuredevops/build.yml`).
2.  **Extract** the `cmake` command line arguments used for the Linux build.
3.  **Filter** for architecture flags (`-damd_gpu_targets`, `-Dgpu_targets`).
4.  **Map** these flags to the Nix `cmakeFlags` array, overriding generic targets with `gfx1151`.