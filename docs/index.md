# theta

Rust CLI for managing agent configurations defined by [theta-spec](https://theta-spec.tamarillo.ai/).

## Installation

```bash
curl -sfL https://raw.githubusercontent.com/tamarillo/theta/main/scripts/install.sh | bash
```

Or build from source:

```bash
cargo install --path crates/theta
```

## Quick start

```bash
theta init                                     # scaffold theta.toml
theta add rule python-types                    # add a rule
theta add tool fetch --command "uvx mcp-fetch" # mcp tool manual addition
theta add skill deploy org/skills@main         # GitHub shorthand
theta check                                    # validate everything
theta cast to claude-code                      # --> CLAUDE.md + .mcp.json + .claude/
```

## What it does

theta reads `theta.toml` and resolves, locks, materializes, and casts agent configurations to any supported harness. Like a package manager but for agent harness resources.

- **Resolves** dependencies (skills, tools, subagents) from local, git, or system store sources
- **Locks** them deterministically into `theta.lock`
- **Materializes** them into `.theta/`
- **Casts** the manifest into any supported harness's native config format
- **Imports** existing harness configs back into `theta.toml`

## Create from harnesses

```bash
cd /path/to/your/project
theta cast from claude-code
```

| Target | Cast | Import |
|--------|------|--------|
| [Claude Code](https://code.claude.com/) | Yes | Yes |
| [Codex CLI](https://github.com/openai/codex) | Yes | Yes |
| [GitHub Copilot](https://code.visualstudio.com/docs/copilot/overview) | Yes | Yes |
| [Cursor](https://cursor.com/) (3+) | Yes | Yes |

### Commands

| Group | Commands |
|---|---|
| **Lifecycle** | `init`, `check`, `lock`, `sync`, `cast to`, `cast from`, `tree` |
| **Dependencies** | `add rule/system/tool/skill/subagent`, `rm rule/system/tool/skill/subagent` |
| **Inspection** | `describe`, `list rules/tools/skills/subagents` |
| **System store** | `register skill/rule/agent`, `list store`, `rm store`, `init --from` |

## Documentation

- [Getting started](getting-started.md) — install, init, first cast
- [Guides](guides/index.md) — tools, skills, casting, system store, git sources
- [Concepts](concepts/index.md) — sources, locking, caching
- [CLI reference](reference/cli.md) — every verb, every flag
- [Settings](reference/settings.md) — environment variables, directory overrides

It is more than recommended to read [theta-spec](https://theta-spec.tamarillo.ai/) first, given that this is the standard that theta implements.

## See also

- [theta-spec](https://theta-spec.tamarillo.ai/) — the standard
- [Agent Skills spec](https://agentskills.io/specification) — skill packaging format
- [MCP](https://modelcontextprotocol.io/) — tool protocol
- [uv](https://github.com/astral-sh/uv) — architectural reference
