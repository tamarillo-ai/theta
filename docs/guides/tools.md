# Managing tools

Tools in `theta.toml` are MCP server declarations. theta supports two modes: explicit configuration and registry lookup.

## Explicit tools

Provide the transport directly:

```bash
# stdio transport
theta add tool fetch --command "uvx mcp-server-fetch"

# HTTP transport
theta add tool api --url "https://api.example.com/mcp"
```

Result in `theta.toml`:

```toml
[tools.fetch]
command = ["uvx", "mcp-server-fetch"]

[tools.api]
url = "https://api.example.com/mcp"
```

### Environment variables

Pass secrets or config via `--env`:

```bash
theta add tool osint --command "uvx osint-mcp" --env OSINT_API_KEY='${env:OSINT_API_KEY}'
```

The `${env:NAME}` syntax defers resolution to the harness runtime — the value is never written to `theta.toml`.

### Extra arguments

```bash
theta add tool legacy --command "node server.js" --args "--verbose" --args "--port=3000"
```

### Disabling tools

```bash
theta add tool experimental --command "uvx new-tool" --disabled
```

Disabled tools are preserved in the manifest but omitted from cast output.

## Registry tools

theta integrates with the [MCP Registry](https://registry.modelcontextprotocol.io/) (conforming to the [MCP Registry specification](https://modelcontextprotocol.io/specification/2025-11-25)) to resolve tools by name:

```bash
theta add tool @anthropic/fetch
theta add tool exa/web-search@1.0.0
```

theta auto-detects registry references (names containing `/` that aren't paths). Resolution:

- Queries the configured registry URL (default: `registry.modelcontextprotocol.io`) for server metadata
- Synthesizes the command from the package type (`npm` --> `npx`, `pypi` --> `uvx`, `oci` --> `docker run`)
- Writes required environment variables as `${env:NAME}` placeholders
- Writes the resolved tool to `theta.toml`

### Pinned versions

```bash
theta add tool @anthropic/fetch@1.2.0     # pinned - cached permanently
theta add tool @anthropic/fetch            # latest - cached for 1 hour
theta add tool @anthropic/fetch --no-cache # bypass cache, always fetch fresh
```

### Custom registries

```bash
theta add tool my-org/internal-tool --registry https://registry.internal.corp/v1
```

### Supported package types

| Registry type | Synthesized command | Runtime hint |
|---|---|---|
| `npm` | `npx -y <identifier>[@version]` | `npx` (default) |
| `pypi` | `uvx <identifier>[==version]` | `uvx` (default), `pip` |
| `oci` | `docker run --rm -i <identifier>` | — |
| `nuget` | `dotnet tool run <identifier>` | — |

### HTTP headers

For remote (SSE/streamable HTTP) tools with auth:

```bash
theta add tool remote-api --url "https://api.example.com/mcp" \
  --header "Authorization=Bearer ${env:API_TOKEN}"
```

## Removing tools

```bash
theta rm tool fetch
```

## Listing tools

```bash
theta list tools
```

Shows name, transport type (stdio/http), target, and enabled/disabled status.
