# AGENTS.md

Root route card for the shared AoA / ToS workspace and the `8Dionysus` public entry repository.

## Purpose

`8Dionysus` is the public route map, profile orientation surface, and selected shared-root install source for the AoA / ToS ecosystem.
Use it to choose the owning repository, audit the AGENTS map, and keep workspace projection surfaces legible.
It is not the constitutional center, not a runtime owner, and not a replacement for layer-owned truth.

## Owner lane

This repository owns:

- public entry orientation, glossary alignment, and profile-level route help
- selected shared-root install sources such as `AGENTS.md`, `AOA_WORKSPACE_ROOT`, `.agents/`, and `.codex/` when checked in here
- workspace bootstrap notes, Codex-plane regeneration notes, and AGENTS map audit surfaces

It does not own:

- AoA center doctrine, which belongs in `Agents-of-Abyss`
- ToS authored meaning, which belongs in `Tree-of-Sophia`
- runtime behavior, which belongs in `abyss-stack`
- SDK helpers, skills, techniques, evals, routing, memory, KAG, playbooks, stats, agents, or seed canon owned by sibling repos

## Start here

1. Choose the primary owner repository before editing.
2. Read `README.md`, `docs/START_HERE.md`, `GLOSSARY.md`, `docs/PUBLIC_ENTRY_POSTURE.md`, and the target repository `README.md` plus `AGENTS.md`.
3. For workspace bootstrap or projection work, also read `docs/WORKSPACE_INSTALL.md`, `docs/CODEX_PLANE_REGENERATION.md`, and the relevant `.agents/` or `.codex/` source surface.
4. For historical detail preserved from the pre-slim root, read `docs/AGENTS_ROOT_REFERENCE.md`.


## AGENTS stack law

- Start with this root card, then follow the nearest nested `AGENTS.md` for every touched path.
- Root guidance owns repository identity, owner boundaries, route choice, and the shortest honest verification path.
- Nested guidance owns local contracts, local risk, exact files, and local checks.
- Authored source surfaces own meaning. Generated, exported, compact, derived, runtime, and adapter surfaces summarize, transport, or support meaning.
- Self-agency, recurrence, quest, progression, checkpoint, or growth language must stay bounded, reviewable, evidence-linked, and reversible.
- Report what changed, what was verified, what was not verified, and where the next agent should resume.

## Decision memory

After a meaningful structural, ownership, workflow, route-law, validator-authority,
public-contract, or topology change, perform a decision review in the owning
repository.

If future agents will need to know why this path was chosen, add or update the
repo-local decision record surface, usually `docs/decisions/`. If no record is
needed, say so in closeout.

## Route by intent

- `Agents-of-Abyss`: ecosystem identity, charter, layer map, federation rules, program direction.
- `Tree-of-Sophia`: source-linked knowledge, texts, concepts, lineages, interpretation architecture.
- `Dionysus`: seed garden, staging, replay, planting trace, early forms.
- `abyss-stack`: runtime, deployment, storage, lifecycle, infrastructure posture.
- `ATM10-Agent`: local companion behavior, perception, retrieval, KAG-in-project, safe operator automation.
- `aoa-sdk`: typed workspace integration, discovery, compatibility, bounded activation helpers.
- `aoa-techniques`: reusable engineering practice.
- `aoa-skills`: bounded execution workflows.
- `aoa-evals`: portable proof and evaluation surfaces.
- `aoa-routing`: navigation and dispatch hints.
- `aoa-memo`: explicit memory and recall objects.
- `aoa-kag`: derived provenance-aware knowledge substrates.
- `aoa-playbooks`: recurring scenario composition, questlines, campaigns, handoffs.
- `aoa-agents`: role contracts, handoff posture, progression and checkpoint contract surfaces.
- `aoa-stats`: derived observability and movement summaries.
- `8Dionysus`: public route map, shared-root projection source, AGENTS map audit.

## Workspace ingress and mutation gate

For substantial workspace-root work, run one ingress pass once the owner repo is chosen:

```bash
aoa skills enter <repo_root> --root <workspace-root> --intent-text "<task>" --json
```

Before risky, mutating, infra, runtime, repo-config, or public-share actions, run:

```bash
aoa skills guard <repo_root> --root <workspace-root> --intent-text "<task>" --mutation-surface <surface> --json
```

Use `aoa surfaces detect` only as additive read-only routing help. It does not overrule owner truth.

## Projection and audit rules

- When shared-root install surfaces change, edit the source-owned copy under `<workspace-root>/8Dionysus/` first, then project them into the live workspace root.
- Do not treat live projected copies at `/AGENTS.md`, `/AOA_WORKSPACE_ROOT`, `/.agents/`, or `/.codex/` as primary truth; in projection wording, do not treat the live copies as source.
- Keep `8Dionysus/README.md` profile-owned and GitHub-facing.
- Treat generated or install drift as a route signal: keep the evidence narrow, route it to the owner repository, and do not promote derived reports into authority.
- Before large AGENTS refactors, run the map audit:

```bash
python scripts/audit_agents_map.py --workspace-root <workspace-root> --write generated/agents_map.min.json --markdown docs/AGENTS_MAP.md
```

For public bootstrap without sibling checkouts:

```bash
python scripts/audit_agents_map.py --public-baseline --write generated/agents_map.min.json --markdown docs/AGENTS_MAP.md
```

After local `AGENTS.md` coverage changes, refresh the frontier report:

```bash
python scripts/recon_agents_frontier.py --map generated/agents_map.min.json --write generated/agents_frontier_recon.min.json --markdown generated/agents_frontier_recon.md
```

## GitHub landing workflow

Root `AGENTS.md` owns the repository-wide branch, PR, CI, and merge route.
`.github/AGENTS.md` owns the GitHub-native files that support it.

When the user asks to commit, push, and merge in this repository, use this route:

1. Start from a branch based on the current `origin/main`. If the worktree is already dirty, inventory it first and carry forward only the intended diff.
2. Commit the intended change with a message that names the changed surface.
3. Push the branch and open a pull request that states changed surfaces, validation run, skipped checks, and remaining risk.
4. Wait for GitHub `Repo Validation` and any required GitHub checks. If a check fails, fix the branch and wait for the new result.
5. Merge through GitHub after green validation. Use squash unless repository settings report a different required method; report the method that landed.
6. Return to `main`, fast-forward from `origin/main`, and confirm the worktree is clean before closeout.

If GitHub status or merge permissions cannot be observed, stop the landing route and report the exact blocker instead of guessing.

## Verify

Use the smallest route-safe check for the changed surface. For AGENTS-map or workspace route changes, run one of the audit commands above and report whether it was a public-baseline or sibling-workspace pass.
If projection, hooks, plugin, convergence, or closeout details are touched, read `docs/AGENTS_ROOT_REFERENCE.md` and run the named narrow helper there before reporting.
For convergence checks, keep `aoa-codex-doctor`, `aoa-codex-status`, and `aoa-codex-bootstrap` discoverable as wrappers, and keep `aoa_codex_convergence_report.{json,md}` as evidence only. The rule is that convergence reports are evidence, not authority.

## Full reference

`docs/AGENTS_ROOT_REFERENCE.md` preserves the previous detailed root guidance, including plugin, hook, convergence, closeout, and projection details.
Use it as a depth layer when the short route card is not enough. If active rules from that reference still govern a local path, prefer moving them to the nearest owner surface rather than bloating this root again.
