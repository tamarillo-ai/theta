---
name: use-theta
description: 'Use the theta CLI to mutate this agent''s configuration: add/remove rules, tools, skills, subagents; cast to and from harnesses (Claude Code, Codex CLI, Cursor, Copilot); validate the manifest; lock and materialize dependencies. USE WHEN: editing theta.toml or any agent-config artifact (instructions, rules, skills, subagents, MCP tools); importing existing harness config; exporting to a harness; checking manifest validity; bumping schema version; reusing resources from the system store. DO NOT USE FOR: writing the actual content of a rule/skill (write the markdown directly), runtime model behavior, MCP server implementation details.'
argument-hint: 'State the mutation you want, e.g. "add a rust rule that applies to *.rs", "cast to claude-code", "import from cursor", "register the osint skill into the system store"'
---

# use-theta

theta is the **mutation engine** for this agent's configuration. The single source of truth is `theta.toml` at the project root. Every change to instructions, rules, tools, skills, or subagents goes through the `theta` CLI — never hand-edit the harness-native files (`CLAUDE.md`, `.cursor/rules/`, `.github/copilot-instructions.md`, `AGENTS.md`, `.mcp.json`, etc.). They are generated.

## Installation

```bash
curl -sfL https://raw.githubusercontent.com/tamarillo-ai/theta/main/scripts/install.sh | bash
```

Requires a Rust toolchain (`rustup`). The installer clones and runs `cargo install`.

## Mental model

```
theta.toml ──(theta lock)──> theta.lock ──(theta sync)──> .theta/ ──(theta cast to <harness>)──> harness-native files
     ▲                                                                                                │
     └────────────────────────(theta cast from <harness>)───────────────────────────────────────────┘
```

- **`theta.toml`** — declarative, harness-agnostic manifest. Conforms to [theta-spec](https://theta-spec.tamarillo.ai/) schema `2026-04`.
- **`theta.lock`** — pins every dependency (git SHA, content hash). Deterministic.
- **`.theta/`** — materialized resources (skills, rules fetched from git/system, subagent manifests). Read by casters.
- **harness files** — generated. Overwriting them by hand will be lost on the next `cast`.

## Core invariants (do not violate)

- **Manifest is source of truth.** If it is not in `theta.toml`, it is invisible to theta.
- **Casting is deterministic.** Same manifest + same target → same output. No LLM calls, no network during cast.
- **Silent loss is forbidden.** When a field has no equivalent in the target harness, `theta` emits a warning; never assume the cast was clean — read the output.
- **Paths in `theta.toml` are relative to the manifest** and **MUST NOT** reference `.theta/` (that directory is generated).
- **Names are kebab-case**: `^[a-z0-9]+(-[a-z0-9]+)*$` for agent/skill/tool/subagent names. Rules MAY use path-qualified kebab-case (`backend/typescript`).

## When to invoke this skill

| Trigger | Action |
|---|---|
| User asks to add a coding standard, style rule, or guardrail | `theta add rule <name>` |
| User asks to add an MCP tool / external server | `theta add tool <name> --command "..."` or `--url "..."` |
| User asks to add a reusable capability package | `theta add skill <name>` (scaffolds `skills/<name>/SKILL.md`) |
| User asks to delegate to a child agent | `theta add subagent <name>` |
| User asks "does this config look right?" / "validate" | `theta check` |
| User wants the agent to run under Claude Code / Cursor / Copilot / Codex | `theta cast to <harness>` |
| User has existing harness config and wants it portable | `theta cast from <harness>` |
| User asks to bump version / change description / set model | `theta describe ...` or edit `[agent]` |
| User asks to inspect what is registered | `theta list rules|tools|skills|subagents|store` |
| User asks to print the manifest schema | `theta schema` |

If the request is to **author content** (e.g., "write a rust style rule"), the workflow is: first run `theta add rule rust --apply glob --apply-to "*.rs"` to register and scaffold the file, then write the markdown into the scaffolded path. Never create the file by hand and forget to register it.

## Operating procedure

### 1. Read state before mutating

Run these before any non-trivial change. They are read-only.

```
theta check
theta list rules
theta list tools
theta list skills
theta list subagents
theta tree
```

`theta check` will surface scaffold-template warnings, missing files, lock staleness, and unresolved sources. Address errors before adding more.

### 2. Mutate via verbs, not text edits

Prefer `theta add` / `theta rm` over hand-editing `theta.toml`. The CLI enforces name patterns, scaffolds source files, and writes TOML with deterministic formatting.

When the CLI cannot express the change (e.g., setting `[harness.<name>]` passthrough fields, or `[extras.<name>]` opaque sections), edit `theta.toml` directly and run `theta check` to confirm validity.

### 3. Lock and sync before casting

`theta cast to <harness>` auto-runs `theta lock` and `theta sync` if needed. Call them explicitly when:

- A `git`-sourced resource was updated upstream and you want to repin — `theta lock --force`.
- `.theta/` looks stale and you want a clean materialize — `theta sync --force`.

### 4. Cast and verify

```
theta cast to claude-code
theta cast to codex-cli
theta cast to cursor
theta cast to copilot
```

Read warnings carefully. Each lossy drop is reported. Use `theta cast to <harness> --notes` to print known per-harness limitations before casting.

`--force` overwrites existing harness files. Default is conflict-refusal — safer.

### 5. Round-trip from an existing harness project

When a project already has Claude Code / Cursor / Copilot / Codex config and the user wants a `theta.toml`:

```
theta cast from claude-code
theta cast from copilot --cross-read   # also imports files discoverable across harness locations
```

Imported portable fields land in standard sections; harness-specific fields land in `[harness.<name>]`. Subagent prompt bodies are externalized to `subagents/<name>.md` and referenced via `prompt_path`.

## Verb cheat-sheet

| Verb | Purpose |
|---|---|
| `theta init` | Scaffold `theta.toml`. `--from <name>` to clone from system store. |
| `theta check` | Validate manifest, lockfile, materialization. `--schema-only` skips materialization checks. |
| `theta describe [DESC]` | Read or set `[agent].description`. `--rules` also prints rule summaries. |
| `theta add rule <name>` | Scaffold rule + register. Flags: `--apply always\|model-decision\|glob\|manual`, `--apply-to "*.rs"`, `--description ...`, `--system <store-name>`, `--path/--git/--file/--branch/--tag/--rev`. |
| `theta add system` | Scaffold `instructions/system.md` + register. |
| `theta add tool <name>` | Register an MCP server. `--command "..."` for stdio or `--url "..."` for HTTP. `--env KEY=VALUE` and `--header KEY=VALUE` repeatable. `<name>` MAY be a registry reference (`io.github.user/tool[@version]`). |
| `theta add skill <name>` | Scaffold or register from `--path`, `--git`, GitHub shorthand (`owner/repo[/subdir][@ref]`), or `--system`. |
| `theta add subagent <name>` | Register a child agent. `--agent-ref <path>` for ref mode, `--prompt-path <path.md>` for inline, `--description-only`. `--model`, `--tools`, `--skills` repeatable. |
| `theta rm <kind> <name>` | Remove from manifest. `--delete` also removes the source file/dir. |
| `theta list <kind>` | List `rules`, `tools`, `skills`, `subagents`, or `store`. |
| `theta lock` | Resolve all sources, write `theta.lock`. `--force` re-locks even when up-to-date. |
| `theta sync` | Materialize locked deps into `.theta/`. `--force` re-syncs everything. |
| `theta cast to <target>` | Export to a harness. `--output <dir>`, `--force`, `--notes`. |
| `theta cast from <source>` | Import from a harness. `--input <dir>`, `--force`, `--cross-read`, `--notes`. |
| `theta register skill\|rule\|agent` | Promote a local resource into the system store. `--force` to overwrite. |
| `theta tree` | Print the subagent dependency tree. |
| `theta schema` | Print the JSON Schema for `theta.toml`. |

## Supported harnesses

| Harness | `theta cast to` target | `[harness.*]` key | Primary output |
|---|---|---|---|
| [Claude Code](https://code.claude.com/) | `claude-code` | `claude_code` | `CLAUDE.md`, `.claude/`, `.mcp.json` |
| [Codex CLI](https://github.com/openai/codex) | `codex-cli` | `codex` | `AGENTS.md`, `.codex/`, `.agents/skills/` |
| [Cursor](https://cursor.com/) (3+) | `cursor` | `cursor` | `.cursor/rules/`, `.cursor/mcp.json`, `.cursor/skills/` |
| [GitHub Copilot](https://code.visualstudio.com/docs/copilot/overview) | `copilot` | `github_copilot` | `.github/copilot-instructions.md`, `.github/instructions/`, `.github/skills/`, `.vscode/mcp.json` |

## Apply modes for rules

| Mode | When | Required fields |
|---|---|---|
| `always` (default) | Rule is always injected into system prompt | — |
| `glob` | Rule is injected when the active file matches a pattern | `apply_to = ["*.rs", ...]` |
| `model-decision` | Model reads `description` and chooses whether to apply | `description = "..."` |
| `manual` | Only applied on explicit user invocation | — |

## Source kinds

```toml
# Local path (relative to theta.toml)
[skills.osint]
source = { path = "skills/osint" }

# Git
[skills.web-design]
source = { git = "https://github.com/vercel-labs/agent-skills", subdirectory = "skills/web-design-guidelines", branch = "main" }

# User's system store (a personal library — `theta register` populates it)
[skills.deploy]
source = { system = "deploy" }
```

`branch`, `tag`, `rev` are mutually exclusive — at most one. SCP-form URLs (`git@host:path`) are rejected; use `https://` or `ssh://`.

## Failure modes and recovery

| Symptom | Diagnosis | Recovery |
|---|---|---|
| `theta check` reports `still using scaffold template` | A scaffolded rule/system file was never filled in | Write real content into the file |
| `theta check` reports unresolved git source | `theta.lock` references a SHA that is no longer fetchable | `theta lock --force` to repin |
| `theta cast to` warns about dropped fields | Field has no equivalent in the target harness | Move the field into `[harness.<name>]` if it is harness-specific, or accept the drop |
| `theta cast to` refuses due to conflicts | Harness file was edited by hand | Inspect the diff. If your edits should be persisted, port them into `theta.toml`. Then `--force`. |
| `.theta/` looks corrupted or stale | Concurrent modification or interrupted sync | `theta sync --force` |
| Schema version mismatch | `theta.toml` uses an older or unknown `schema` value | Migrate per the message; current is `"2026-04"` |

## References

- [theta CLI docs](https://theta.tamarillo.ai/)
- [theta-spec](https://theta-spec.tamarillo.ai/) — the declarative standard
- [Agent Skills spec](https://agentskills.io/specification)
- [MCP](https://modelcontextprotocol.io/)
