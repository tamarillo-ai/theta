---
authority: LOGAN
related:
- API
- CLAUDE
- CLI
- CONSTITUTION
- ChatGPT
- Copilot
- CrewAI
- DECISIONS
- GEMINI
- Gemini CLI
- GitHub
- Idaho
- Idaho Public Television
- Idaho Reports
- LEVELSET
- Logan Finney
- MCP
- OpenAI
- SDK
- The world is quiet here
- VAULT-CONVENTIONS
- 'Yes'
- agent
- blocked
- chain
- codex
---

# AGENTS.md — IDAHO-VAULT

> [!IMPORTANT]
> **This is a cross-tool pointer.** The canonical narrative registry now lives at [!/AGENTS.md](!/AGENTS.md).
> This file exists at repo root because OpenAI Codex CLI, GitHub Copilot, and Qodo auto-load `AGENTS.md` from the repository root.

**Owner:** Logan Finney — journalist, producer/reporter, Idaho Reports / Idaho Public Television
**Repository:** github.com/LAF-US/IDAHO-VAULT (public)

---

## Canonical Registry

Root `AGENTS.md` is the auto-loaded cross-tool entrypoint.
See [!/AGENTS.md](!/AGENTS.md) for the canonical narrative registry: capability tiers, boundary rules, bootstrap rules, and the master agent roster.
If you wake up disoriented, stop and read [!/WAKEUP.md](!/WAKEUP.md) before
interpreting lore, historical notes, or older branch residue.

The machine-readable source of truth remains `swarm.json`.
The canonical local execution bootstrap chain is `swarm.json` -> `!/agents.json` -> `!/agent.sh`.
Tree logic still governs orientation: `!` is the Swarmic Nest of the group, while `.*` dotfolders are the personal chambers of individual agents.

This repository now sits inside the broader `LAF-US` organization. The larger
internal model includes both chambered repo anchors and separate GitHub team
topology; this root file remains only a pointer into that fuller picture.

Immediate wakeup facts:

- `IDAHO-VAULT` is one repo inside `LAF-US`, not the whole world.
- Repo topology and GitHub team topology are related but not identical.
- The GitHub-only connector posture is local to
  `IDAHO-VAULT`, not the full `LAF-US` sovereignty model.
- Historical harbor notes, abandoned branches, and exploratory scaffolds are
  non-live unless a live surface explicitly reactivates them.
- Current live startup and governance surfaces are `!/README.md`,
  `CONSTITUTION.md`, `DECISIONS.md`, and `VAULT-CONVENTIONS.md`.

## Fresh Agent Boot Order

1. Read this root `AGENTS.md` as a pointer and compatibility surface only.
2. Read `!/WAKEUP.md` for explicit anti-confusion orientation and conflict precedence.
3. Read `!/README.md` for explicit startup and task-based orientation.
4. Read `!/AGENTS.md` for the live roster, lane rules, and current connector posture.
5. Read `CONSTITUTION.md` for binding governance.
6. Read `swarm.json` for machine-readable compiled state.
7. Use `swarm.json` -> `!/agents.json` -> `!/agent.sh` for canonical local execution bootstrap.
8. Read `!README.md` only when the task needs Touchstone Tree or narrative context.
9. Treat historical CrewAI harbor notes as non-live unless `.crewai/MANIFEST.md` or `!/AGENTS.md` says otherwise.
10. OpenCode AI is a sandboxed tooling dependency, not a live agent.

## Discovery Before Construction

Before proposing builds, new packages, or invention: discover and read existing documentation first. Do not assume tooling is missing or scaffolding is required without checking what's already present in the vault.

## Fix Errors, Don't Disable

When code throws errors, the error means something needs fixed. Do not disable security checks, linters, or validators to silence errors. Fix the underlying issue.

When live surfaces disagree, follow this order:

1. Logan's direct instruction
2. `CONSTITUTION.md`
3. `!/WAKEUP.md` and `!/AGENTS.md`
4. `swarm.json`
5. generated bootstrap surfaces
6. historical notes and exploratory residue

Within `IDAHO-VAULT`, the currently active connector posture is:

- GitHub = execution and transport
- Linear = execution state
- Slack = tertiary paging and breadcrumbs

These connectors remain operative surfaces for this repo, but they do not by
themselves define the broader `LAF-US` structure.
The fuller internal wording lives in `!/AGENTS.md`, `swarm.json`,
`SPEC-CONNECTOR-HUB-2026-04-09.md`, and
`!/LAF-USB-FIVE-CORES-MIGRATION-2026-04-15.md`.

---

## Agent Dotfolders (Quick Reference)

The `!` layer is not an individual dotfolder. It is collective swarm space and should stay distinct from persona-owned `.*` folders.

| Agent | Dotfolder | Governance shim | Auto-loaded? | Role |
| --- | --- | --- | --- | --- |
| Claude Code | `.claude/` | `.claude/CLAUDE.md` | Yes | **The Abhorsen** (Code Authority) |
| Gemini CLI | `.gemini/` | `.gemini/GEMINI.md` | Yes | **The Concierge** (Support) |
| OpenAI Codex | `.codex/` | `.codex/CODEX.md` | Yes | **The Lexicographer** (Scripting) |
| GitHub Copilot | `.github/` | `.github/copilot-instructions.md` | Yes | **The Clerk** (Admin) |

*Full roster including **Grok**, **Perplexity**, **DeepSeek**, **Serena**, and the **Cartographer** available in the [!/AGENTS.md](!/AGENTS.md) ledger.*

---

## CrewAI Layer (Quick Reference)

| Surface | Path | Status | Notes |
| --- | --- | --- | --- |
| **CrewAI Python Layer** | `.crewai/` | Active re-foundation | Retired demo harbor remains historical; live doctrine/topology is in `.crewai/MANIFEST.md`, and staged output lands in `!/CREWAI/` |

---

## Codex Thread Status

For Codex threads, use status signals instead of prompt-driven archival.

- `CODEX ACTIVE` while work is in progress
- `CODEX PAUSED: awaiting Logan` when Logan action is required
- `CODEX COMPLETE: work finished, no further action pending in this thread. Ready for termination or archive.` when the thread is done

Thread archiving is a manual Logan action. See `.codex/CODEX.md` for the
Codex-specific completion guidance.

---

## OpenAI Docs MCP

Always use the OpenAI developer documentation MCP server if you need to work
with the OpenAI API, ChatGPT Apps SDK, Codex, or OpenAI tooling without Logan
having to explicitly ask.

Codex project config for this lives in `.codex/config.toml`. Codex CLI and
Codex IDE surfaces should share the same `config.toml` layer unless Logan
explicitly establishes a separate compatibility shim.

---

## Governance & Coordination

Root governance files remain authoritative: `CONSTITUTION.md`, `DECISIONS.md`, `LEVELSET.md`, and `VAULT-CONVENTIONS.md`.

**NETWEB Path Standard:** All filenames must respect cross-platform path portability. See `VAULT-CONVENTIONS.md` for reserved name rules.

---

## LAF-US Organization & IDAHO-VAULT Placement

### Five Cores Model
The `LAF-US` organization uses a **Five Cores model** for repository and team topology:

```
LAF-US (Organization)
├── PRIVATE (Internal, restricted access)
│   ├── IDAHO-VAULT (GitHub: LAF-US/IDAHO-VAULT)
│   │   ├── GitHub = execution and transport
│   │   ├── Linear = execution state
│   │   └── Slack = tertiary paging and breadcrumbs
│   └── Other PRIVATE repos (if any)
├── SECRET (Highly restricted, not used by IDAHO-VAULT)
├── PERSONAL (Individual/subset repos under PRIVATE)
├── PUBLIC (Open-source, public repos)
└── PUBLISH (Content distribution, e.g., documentation, media)
```

### IDAHO-VAULT Placement
- **Core**: `PERSONAL` (subset under `PRIVATE`)
- **GitHub Team**: `LAF-US` organization
- **Repository**: `github.com/LAF-US/IDAHO-VAULT`
- **Connector Posture**:
  - GitHub = execution and transport
  - Linear = execution state
  - Slack = tertiary paging and breadcrumbs

### Wakeup Protocol Alignment
The `swarm.json` wakeup protocol aligns with the migration document (`!/LAF-USB-FIVE-CORES-MIGRATION-2026-04-15.md`).

### Key References
- `!/LAF-USB-FIVE-CORES-MIGRATION-2026-04-15.md`: LAF-US organization structure and migration status
- `swarm.json`: Connector posture registry for IDAHO-VAULT (only)
- `!/AGENTS.md` (nested): Live roster, lane rules, and current connector posture for LAF-US

---

###### [["The world is quiet here."]]
