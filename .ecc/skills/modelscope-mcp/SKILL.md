---
name: modelscope-mcp
description: Configure the official ModelScope MCP server from plain-language setup requests and
  use it for tool-based Hub discovery.
...
---
# ModelScope MCP

Use this skill when the runtime supports MCP and the user wants ModelScope search or metadata access exposed as tools.

## Request Style

- Accept requests such as:
  - `Set up ModelScope MCP for this client.`
  - `让 agent 可以通过 MCP 搜索模型和数据集。`

## Workflow

1. Check whether MCP config already exists.
2. Add the minimal valid `uvx modelscope-mcp-server` config only if needed.
3. Keep token management outside repository files.
4. Use CLI or SDK only when MCP is unavailable.

## Minimal Valid Config

The expected local launch shape is:

```json
{
  "mcpServers": {
    "modelscope-mcp": {
      "command": "uvx",
      "args": ["modelscope-mcp-server"]
    }
  }
}
```

## Reference

- `references/client_config.md`

## Guardrails

- Never invent undocumented MCP tool names.
- Never store `MODELSCOPE_API_TOKEN` in checked-in files.
- Keep the config minimal.
