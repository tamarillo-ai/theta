# Workspace Plugins

This directory holds workspace-scoped local Codex plugins for `/srv/AbyssOS`.

Boundary order:

1. owner-repo skill and role meaning
2. workspace MCP authority in `/srv/AbyssOS/.codex/config.toml`
3. plugin discovery in `/srv/AbyssOS/.agents/plugins/marketplace.json`
4. plugin packaging in `/srv/AbyssOS/.codex/plugins/`

Do not treat installed plugins here as owner truth.
They are launcher packaging that depends on already-landed workspace surfaces.
