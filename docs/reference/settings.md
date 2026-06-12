# Settings

theta resolves settings using a cascade: CLI flag > environment variable > built-in default.

## Directory overrides

| Setting | CLI flag | Environment variable | Default |
|---|---|---|---|
| Instructions directory | `--instructions-dir` | `THETA_INSTRUCTIONS_DIR` | `instructions` |
| Rules subdirectory | `--rules-dir` | `THETA_RULES_DIR` | `rules` |
| Subagent prompts directory | `--subagent-prompts` | `THETA_SUBAGENTS_DIR` | `subagents` |

## Output directory override

| Environment variable | Effect |
|---|---|
| `THETA_OUT_DIR` | When set, `theta sync` and `theta lock` write `.theta/` and `theta.lock` to this directory instead of next to `theta.toml`. Source files are still resolved relative to the manifest. Useful for materializing into a temporary directory without modifying the source tree — for example, when building tooling that reads project content without side effects. |

The instructions/rules settings control where `theta add system` and `theta add rule` scaffold files. `THETA_SUBAGENTS_DIR` controls where `theta cast from` writes externalized subagent prompt files and where `theta add subagent` scaffolds new `.md` files:

```
<instructions-dir>/
├── system.md
└── <rules-dir>/
    ├── safety.md
    └── style.md
```

With defaults, that's `instructions/system.md` and `instructions/rules/safety.md`.

## Well-known paths

| Path | Purpose |
|---|---|
| `theta.toml` | Agent manifest |
| `theta.lock` | Lockfile (committed to VCS) |
| `.theta/` | Materialized dependencies (gitignored) |
| `~/.cache/theta/` | Cache (XDG: `$XDG_CACHE_HOME/theta/`) |
| `~/.cache/theta/git/` | `git` clone cache |
| `~/.cache/theta/registry/` | MCP registry response cache |
| `~/.local/share/theta/store/` | [System store](../guides/store.md) (XDG: `$XDG_DATA_HOME/theta/store/`) |

## Manifest resolution

theta finds the manifest by:

1. `--manifest <path>` flag (if provided)
2. `theta.toml` in the current directory (or `--directory` if set)

## Machine-readable output

`theta check`, `theta list`, `theta tree`, `theta sync`, `theta lock`, and `theta get` support `--output-format json`:

```bash
theta check --output-format json
theta list rules --output-format json
theta tree --output-format json
theta sync --output-format json
theta get --output-format json
```

`theta schema` flags:

```bash
theta schema                  # full JSON Schema for theta.toml
theta schema --list-verbs     # verb tree (for tooling / codegen)
theta schema --get            # JSON Schema for theta get output
theta schema --constants      # theta-static path constants (for codegen)
```

`theta get` emits the full materialized project state as JSON — agent identity, lock hash, system prompt, rules, skills (with SKILL.md content and supporting files), and tools. Requires `theta sync` to have run first.
