# System store

The system store is a personal library of reusable rules, skills, and agent templates at `~/.local/share/theta/store/`.

## Register

```bash
theta register rule safety           # register from current theta.toml
theta register skill osint           # register a skill from current project
theta register agent                 # register the entire agent config
theta register agent --name my-agent # override the stored name
```

Registration copies the resource into the store and updates `~/.local/share/theta/store/index.toml`.

For skills, theta also accepts direct sources (no `theta.toml` required):

```bash
theta register skill osint --path ./skills/osint
theta register skill deploy-to-vercel --git https://github.com/vercel-labs/agent-skills --subdirectory skills/deploy-to-vercel
theta register skill deploy-to-vercel vercel-labs/agent-skills/skills/deploy-to-vercel@main
```

Use `--force` to overwrite an existing entry.

## Use from store

Once registered, resources are available as `system` sources:

```bash
theta add rule safety --system       # adds src = { system = "safety" }
theta add skill osint --system       # adds source = { system = "osint" }
theta init --from my-agent           # scaffold a new project from a stored agent
```

`--system` sources are resolved from the store during `theta lock` and materialized into `.theta/` during `theta sync`.

## Inspect

```bash
theta list store                     # show all registered resources
```

Output groups by type (agents, skills, rules) with names and descriptions.

## Remove

```bash
theta rm store skill osint
theta rm store rule safety
theta rm store agent my-agent
```

## Store layout

```
~/.local/share/theta/store/
├── index.toml
├── agents/
│   └── my-agent/
│       ├── theta.toml
│       ├── theta.lock
│       └── instructions/
├── skills/
│   └── osint/
│       └── SKILL.md
└── rules/
    └── safety.md
```
