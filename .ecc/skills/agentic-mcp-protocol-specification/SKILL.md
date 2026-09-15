---
name: agentic-mcp-protocol-specification
description: "Client-Host-Server topology, JSON-RPC 2.0 framing, resource subscriptions, tool invocation contracts, prompt templates, and security sandboxing."
triggers: ["mcp", "model-context-protocol", "json-rpc", "mcp-server", "mcp-client", "tool-invocation", "resource-subscriptions"]
---

# agentic-mcp-protocol-specification
> Based on **Anthropic Model Context Protocol Specification - Anthropic MCP Architecture**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Client-Host-Server Architecture: Host application (e.g. Tagisan) coordinates MCP Clients that connect to isolated MCP Servers providing tools and resources.**
2. **JSON-RPC 2.0 Framing: Strict request, response, notification, and error objects with deterministic error codes (-32600 to -32603).**
3. **Resource URI Schemes: Resources identified by standardized URIs (e.g. `file:///`, `postgres://`) with subscription notifications on content changes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Expose all agent capabilities, tools, and project contexts through the Model Context Protocol (MCP). Enforce strict parameter validation on all incoming tool calls.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing proprietary ad-hoc tool execution protocols when standard MCP provides universal interop.**
- **Failing to validate tool input arguments against the declared JSON schema before execution.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "mcp"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
