# Caching

theta caches remote resources to avoid redundant fetches. All caches live under `~/.cache/theta/` (XDG: `$XDG_CACHE_HOME/theta/`).

## Git cache

```
~/.cache/theta/git/
├── db/           # bare clones, keyed by url digest
├── checkouts/    # working trees, keyed by commit SHA
└── locks/        # file locks for concurrent access
```

Bare clones are reused across projects. When `theta lock` resolves a git source, it fetches only new commits into the existing bare clone.

See [git sources](../guides/git-sources.md) for usage details.

## Registry cache

```
~/.cache/theta/registry/
└── <server-name>/
    ├── <version>.json    # pinned version — cached permanently
    └── _latest.json      # latest lookup — cached for 1 hour
```

When `theta add tool` resolves from the [MCP Registry](https://registry.modelcontextprotocol.io/), the server metadata is cached:

- **Pinned versions** (`@1.2.0`) — never expire
- **Latest** (no version) — 1 hour TTL

## Shared across projects

Both caches are global. Two projects using the same git repo or registry server share the same cached data. No per-project duplication.

## Clearing the cache

```bash
rm -rf ~/.cache/theta/
```

theta recreates cache directories as needed.
