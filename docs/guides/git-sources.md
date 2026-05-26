# Git sources

Rules and skills can be sourced from git repositories. theta fetches, pins, and caches them.

## GitHub shorthand

The fastest way to add remote resources:

```bash
theta add skill owner/repo@ref
theta add skill owner/repo/subdirectory@ref
theta add skill tamarillo/skills/osint@main
```

Expands to `{ git = "https://github.com/owner/repo", rev = "main", subdirectory = "osint" }`.

The `@ref` part maps to `rev` since the shorthand cannot distinguish branches from tags.

## Explicit git flags

For non-GitHub repos or when you need full control:

```bash
# remote skill with a tag
theta add skill deploy-to-vercel --git https://github.com/vercel-labs/agent-skills --tag v2.0 --subdirectory skills/deploy-to-vercel

# remote rule pinned to a branch
theta add rule review --git https://github.com/org/rules --branch main --file review.md

# remote skill pinned to a commit
theta add skill react-best-practices --git https://github.com/vercel-labs/agent-skills --rev abc1234 --subdirectory skills/react-best-practices
```

Exactly one of `--branch`, `--tag`, or `--rev` **MAY** be specified. They are mutually exclusive.

## Resolution pipeline

- **`theta lock`** — fetches the repo, pins the commit SHA in `theta.lock`
- **`theta sync`** — checks out the pinned commit, copies into `.theta/skills/` or `.theta/rules/`
- **`theta cast to`** — reads from `.theta/`

The chain runs automatically on `theta cast to` or `theta sync`.

### Sync behavior

Skills and rules have opposite defaults for syncing after add:

- **Skills** sync by default. Use `--no-sync` to defer:

```bash
theta add skill vercel-labs/agent-skills/skills/web-design-guidelines@main --no-sync
```

- **Rules** do not sync by default. Use `--sync` to trigger immediate resolution:

```bash
theta add rule review --git https://github.com/org/rules --branch main --file review.md --sync
```

## Git cache

theta caches git repos at `~/.cache/theta/git/` with a 3-tier layout:

```
~/.cache/theta/git/
├── db/           # bare clones (keyed by url digest)
├── checkouts/    # working trees (keyed by commit SHA)
└── locks/        # file locks for concurrent access
```

The cache is shared across all projects. Fetches reuse existing clones — only new commits are fetched.

!!! note
    The cache follows XDG conventions. Override with `$XDG_CACHE_HOME`.

## What gets pinned

| Source field | Locked to |
|---|---|
| `branch = "main"` | Exact commit SHA at lock time |
| `tag = "v1.0.0"` | Exact commit SHA of the tag |
| `rev = "abc1234"` | The commit itself |

Run `theta lock --force` to re-resolve refs to their latest commits.
