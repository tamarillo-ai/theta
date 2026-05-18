# Concepts

theta operates usually on a four-stage pipeline:

```
theta.toml --> theta.lock --> .theta/ --> harness config
 (manifest)    (lock)      (sync)    (cast)
```

Each stage is deterministic and independently inspectable.

`theta.toml` is the single source of truth for an agent. For the full field specification, see [theta-spec](https://theta-spec.tamarillo.ai/manifest/).

| Concept | What it is |
|---|---|
| [Sources](sources.md) | Where dependencies come from: path, git, system, url, registry |
| [Lockfile & sync](lockfile-sync.md) | `theta.lock` pins versions; `theta sync` materializes `.theta/` |
| [Caching](caching.md) | Git and registry cache layout, TTL, sharing across projects |
