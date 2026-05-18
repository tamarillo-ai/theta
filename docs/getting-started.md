# Getting started

## Install

## Create an agent

```bash
mkdir my-agent && cd my-agent
theta init
```

This creates `theta.toml` with defaults derived from the directory name and `git config` if set:

```toml
[theta]
schema = "2026-04"

[agent]
name = "my-agent"
description = "Add your description here"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
```

Set a real description:

```bash
theta describe "automatic research tool"
```

## Add instructions

```bash
theta add system                     # scaffolds instructions/system.md
theta add rule safety                # scaffolds instructions/rules/safety.md
theta add rule rust --apply glob --apply-to "*.rs"
```

Fill these files with relevant info. Requirements vary depending on harness — see [casting to harnesses](guides/casting.md) for details.

> Note: `$(pwd)/instructions` will be the host directory for all the instructions files unless you edit `THETA_INSTRUCTIONS_DIR`.


## Add tools and skills

MCP tools and skills are crucial in any modern setting.

```bash
theta add tool fetch --command "uvx mcp-fetch" # hardcoding the command
theta add skill mem-mgmt # scaffolds skills/mem-mgmt.md/SKILL.md and helper optional folders for extra resources
theta add skill vercel-labs/agent-skills/skills/react-native-skills # fetch skills from Vercel
```

It is expected for the `mem-mgmt` skill to be filled with relevant info.

## Validate

To guarantee sanity of the manifest `theta.toml` file, a check can always be run:

```bash
theta check
```

This runs structural validation, reachability checks, deterministic content quality and constraints checks, and lock/materialization consistency.

If you did not fill in any of the generated rules or skills you will get some warnings like the ones displayed below:

```bash
warn [agent].description still the placeholder — edit it to describe your agent
warn [instructions].system system prompt still using scaffold template — edit instructions/system.md
warn [instructions.rules] [instructions.rules.rust] still using scaffold template
warn [instructions.rules] [instructions.rules.safety] still using scaffold template
hint theta.lock no lockfile found — run `theta sync` to lock and materialize dependencies
hint .theta/ not materialized — run `theta sync` to populate
ok theta.toml is valid with 4 warning(s)
```

## Cast to a harness

The manifest provides a harness-agnostic definition of the resources that our agent is going to use. theta provides functionality for both importing and exporting the manifest configuration to harness-specific resources.

```bash
theta cast to claude-code            # --> CLAUDE.md, .mcp.json, .claude/
theta cast to copilot                # --> .github/copilot-instructions.md, .vscode/mcp.json
theta cast to cursor                 # --> .cursor/rules/, .cursor/mcp.json
```

Cast auto-syncs: it locks, materializes, then produces the harness-specific files. You can always use `--force` to overwrite existing configs.

## Import from a harness

Already have a project with Claude Code config?

```bash
theta cast from claude-code          # reads CLAUDE.md, .mcp.json, .claude/
```

This produces a `theta.toml` with portable fields plus a `[harness.claude_code]` section for anything harness-specific.

## Next steps

- [Guides](guides/index.md) — tools, skills, casting, system store, git sources
- [Concepts](concepts/index.md) — sources, locking, caching
- [CLI reference](reference/cli.md) — every command and flag
- [System store](guides/store.md) — reuse rules, skills, and agents across projects
- [Managing tools](guides/tools.md) — MCP registry, env vars, headers
