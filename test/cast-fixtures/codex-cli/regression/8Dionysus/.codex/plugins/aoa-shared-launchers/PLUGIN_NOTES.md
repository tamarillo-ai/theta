# AoA Shared Launchers plugin notes

This plugin is intentionally narrow.

- It bundles shared launcher skills.
- It relies on project-scoped named MCP servers already configured in `.codex/config.toml`.
- It does not try to move AoA owner meaning into the plugin layer.
- It does not bundle `.mcp.json` in this first wave.

Bundled skills:
- `aoa-workspace-recon`
- `aoa-growth-snapshot`
- `aoa-seed-route-inspect`
- `aoa-config-doctor`

Each bundled skill includes `agents/openai.yaml` with named MCP dependencies where appropriate.
`aoa-config-doctor` now resolves the real AoA workspace root from either
`/srv/AbyssOS` or a repo-local checkout before it recommends workspace validators.
