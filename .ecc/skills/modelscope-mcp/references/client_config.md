# Client Configuration Notes

## Cursor (`.mcp.json`)

Use local stdio mode:

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

## Gemini Extension (`gemini-extension.json`)

Keep the same server name and command/args so generated artifacts stay consistent across clients.

## Authentication

Provide `MODELSCOPE_API_TOKEN` via shell environment before starting your MCP client session.
