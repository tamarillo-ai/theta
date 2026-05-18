# Lockfile & sync

## theta.lock

`theta lock` reads `theta.toml` and produces `theta.lock` — a file that pins every dependency to an exact version, commit, or content hash.

```bash
theta lock              # resolve and write theta.lock
theta lock --force      # re-lock even if up to date
```

Properties:

- **Deterministic**: same `theta.toml` --> same `theta.lock`. No LLM calls, no network non-determinism.
- **Committed to VCS**: `theta.lock` goes in version control
- **Staleness detection**: theta tracks a manifest hash. If `theta.toml` changes, the lock is stale and re-resolved automatically.

### Locked source types

| Source | Pinned to |
|---|---|
| path | Content hash |
| git | Commit SHA + content hash |
| system | Content hash |

## .theta/

`theta sync` reads `theta.lock` and populates `.theta/` with materialized dependencies:

```bash
theta sync              # materialize from lockfile
theta sync --force      # re-sync even if up to date
```

Layout:

```
.theta/
├── system.md                          # resolved system prompt
├── rules/
│   ├── safety.md
│   └── typescript.md
├── skills/
│   ├── code-review/
│   │   └── SKILL.md
│   └── deploy/
│       └── SKILL.md
└── subagents/
    ├── scout/                         # ref subagent
    │   ├── theta.toml
    │   ├── system.md
    │   └── rules/
    │       └── recon-style.md
    └── reviewer/                      # inline subagent (prompt_path)
        └── system.md
```

`.theta/` is a derived artifact — add it to `.gitignore`.

## Auto-sync

`theta cast to` auto-syncs before casting. The chain is:

```
theta cast to <target>
  --> theta lock (if stale)
  --> theta sync (if stale)
  --> produce harness files
```

## Validation

After sync, theta validates:

- Every skill directory contains a valid `SKILL.md` with `name` and `description`
- Every rule file exists and is non-empty
- Lock entries match materialized content (consistency check)

`theta check` runs the same validation without modifying anything.
