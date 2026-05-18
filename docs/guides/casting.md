# Casting to harnesses

## Cast to a harness

```bash
theta cast to claude-code
theta cast to copilot --output ./build
theta cast to cursor --force
```

Casting auto-syncs: `theta lock` --> `theta sync` --> produce harness files. The chain runs automatically.

### Output files per target

| Target | Output files |
|---|---|
| `claude-code` | `CLAUDE.md`, `.mcp.json`, `.claude/settings.json`, `.claude/rules/*.md`, `.claude/agents/*.md` |
| `codex-cli` | `AGENTS.md`, `.codex/config.toml`, `.codex/agents/*.toml` |
| `copilot` | `.github/copilot-instructions.md`, `.github/instructions/*.instructions.md`, `.github/agents/*.agent.md`, `.github/skills/*/SKILL.md`, `.vscode/mcp.json`, `.vscode/settings.json`, `.github/hooks/theta-hooks.json` |
| `cursor` | `.cursor/rules/*.mdc`, `.cursor/mcp.json`, `.cursor/agents/*.md` |

### Overwrite protection

By default, `theta cast to` refuses to overwrite existing harness config files:

```
conflict .github/copilot-instructions.md already exists
error: cast would overwrite 1 existing file(s) — use --force to overwrite
```

!!! note "Copilot JSON merge"
    `cast to copilot` reads existing `.vscode/settings.json` and `.vscode/mcp.json` before writing. theta-owned keys overwrite; unrelated user keys are preserved. This avoids destroying VS Code settings not managed by theta. Markdown files (instructions, rules, agents, skills, hooks) are fully overwritten.

### Lossy casting

Not every harness supports every feature. When a field has no equivalent in the target, theta emits a warning and drops it. Lossy casting never fails — it produces the best possible output and tells you what was lost.

### NFR limits

After producing cast output, theta checks harness-specific constraints:

| Harness | Constraint | Level |
|---|---|---|
| Codex CLI | 32K bytes per file | Hard (error) |
| Cursor | 500 lines per rule file | Soft (hint) |
| Claude Code | 200 lines per rule file | Soft (hint) |

### Multi-harness config

A single `theta.toml` can contain `[harness.<name>]` sections for multiple targets:

```toml
[harness.claude_code]
permissions = { allow = ["Bash(npm run *)", "Read(~/.zshrc)"], deny = ["Bash(curl *)"] }

[harness.cursor]
composerMode = "agent"
```

When casting to target X, theta reads the common sections plus `[harness.X]` and ignores the rest.

## Import from a harness

```bash
theta cast from claude-code
theta cast from copilot --input ./project
theta cast from cursor --force
```

Import reads existing harness-native config and produces a `theta.toml`:

- Portable fields --> common sections (`[agent]`, `[tools.*]`, etc.)
- Harness-specific fields --> `[harness.<name>]`
- Content files (system prompt, rules) --> extracted alongside `theta.toml`
- Subagent prompt bodies --> externalized to `subagents/<name>.md` with `prompt_path` in the manifest

### Subagent prompt externalization

During import, subagent bodies (e.g., `.claude/agents/*.md`, `.github/agents/*.agent.md`) are written to `<project>/subagents/<name>.md`. The generated `theta.toml` references them via `prompt_path`.

```bash
theta cast from copilot                               # writes to subagents/
theta cast from claude-code --subagent-prompts ./prompts  # custom directory
theta cast from codex-cli --force-overwrite            # overwrite existing files
```

- `--subagent-prompts <DIR>` overrides the default `subagents/` directory
- `--force-overwrite` replaces existing prompt files with different content (without it, import errors on conflicts)

### Harness version detection

During `theta check` and `theta cast to`, theta detects installed harness versions and validates against version constraints declared in `[harness.<name>]`:

- **No version key** --> hint to consider pinning
- **Version declared but detection fails** --> hint that the harness is undetectable
- **Version mismatch** --> warning
