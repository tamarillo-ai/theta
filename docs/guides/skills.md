# Working with skills

Skills are composable capability packages following the [Agent Skills spec](https://agentskills.io/specification). Each skill is a directory containing a `SKILL.md` file with frontmatter metadata.

## Scaffolding a local skill

```bash
theta add skill code-review
```

Creates:

```
skills/code-review/
├── SKILL.md
├── scripts/
├── references/
└── assets/
```

The generated `SKILL.md`:

```markdown
---
name: code-review
description: "Add a description for the code-review skill"
---

# code-review

Describe what this skill does and when to use it.
```

Fill in the description and body — `theta check` warns if the template is unchanged.

## Adding from a local directory

```bash
theta add skill code-review --path ./skills/code-review
```

Registers the existing directory without scaffolding.

## Adding from GitHub

theta supports a compact shorthand for GitHub-hosted skills:

```bash
theta add skill owner/repo@ref
theta add skill owner/repo/subdirectory@ref
theta add skill tamarillo/skills/osint@main
```

This expands to:

```toml
[skills.react-native-skills]
source = { git = "https://github.com/vercel-labs/agent-skills", branch = "main", subdirectory = "skills/react-native-skills" }
```

The long form works too:

```bash
theta add skill deploy --git https://github.com/org/skills --branch main --subdirectory deploy
```

### Resolution

Remote skills are fetched during `theta sync`:

- `theta lock` pins the git commit SHA in `theta.lock`
- `theta sync` clones to `~/.cache/theta/git/`, checks out, copies the skill into `.theta/skills/`
- Cast reads from `.theta/`

Use `--no-sync` to add without fetching immediately:

```bash
theta add skill deploy org/skills/deploy@main --no-sync
```

## Adding from the system store

```bash
theta add skill osint --system
```

See [system store](store.md) for how to register skills.

## SKILL.md format

The [Agent Skills spec](https://agentskills.io/specification) defines the structure. theta requires:

- **`name`** in YAML frontmatter — kebab-case identifier
- **`description`** in YAML frontmatter — what the skill does

Optional frontmatter fields: `version`, `author`, `tags`.

The markdown body is injected into the agent context by the harness. Structure it as instructions the agent should follow when using this skill.

## Listing skills

```bash
theta list skills
```

Shows name, source type (path/git/system), and source reference.

## Removing skills

```bash
theta rm skill code-review             # remove from manifest
theta rm skill code-review --delete    # also delete the source directory
```

`--delete` removes the local source directory for path-based skills. For git-sourced and system-store skills, `--delete` has no effect — these are resolved from remote or store sources, not local directories. The lock entry and `.theta/` artifact are always cleaned up regardless.
