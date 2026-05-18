# Sources

Skills, rules, and subagents can come from five source types.

## Source types

| Source | Syntax | Use case |
|---|---|---|
| **path** | `{ path = "./local/dir" }` | Local development, workspace-local assets |
| **git** | `{ git = "https://...", tag = "v1.0" }` | Pinned remote repositories |
| **url** | `{ url = "https://...file.md" }` | Raw files from the internet |
| **registry** | `{ registry = "smithery", name = "pkg" }` | Named registries (future) |
| **system** | `{ system = "name" }` | Personal [system store](../guides/store.md) |

## GitHub shorthand

For git-hosted skills, theta supports a compact syntax:

```bash
theta add skill owner/repo@ref
theta add skill owner/repo/subdirectory@ref
theta add skill tamarillo/skills/osint@main
```

This expands to `{ git = "https://github.com/owner/repo", subdirectory = "...", rev = "..." }`.

## Source types per resource

| Resource | path | git | url | registry | system |
|---|---|---|---|---|---|
| Skills | yes | yes | yes | future | yes |
| Rules | yes | yes | — | — | yes |
| Subagents (ref) | yes | — | — | — | — |

Rules use `src` (a `LocalOrGitRef`) instead of `source` (a `SourceRef`). The rule `src` field accepts a plain string (local path), a git table, or a system table — but not url or registry.

## Resolution

Sources are resolved during `theta lock`:

- **path**: verified to exist, content-hashed
- **git**: fetched to `~/.cache/theta/git/`, commit SHA pinned
- **system**: resolved from `~/.local/share/theta/store/`, content-hashed
- **url**: fetched (future)
- **registry**: resolved (future)
