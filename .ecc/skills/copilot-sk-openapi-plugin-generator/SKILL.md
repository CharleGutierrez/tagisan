---
name: copilot-sk-openapi-plugin-generator
description: "Importing OpenAPI specifications dynamically into Semantic Kernel plugins at runtime."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-openapi-plugins", "dynamic-openapi-import", "openapi-to-sk-plugin", "swagger-sk-integration"]
---

# copilot-sk-openapi-plugin-generator
> Based on **RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Dynamic Generation: `kernel.ImportPluginFromOpenApiAsync()` compiles endpoints into callable KernelFunctions.**
2. **Auth Delegation: Inject custom `HttpClient` with Bearer token authentication handlers into the OpenAPI plugin.**
3. **MANDATORY filtering of sensitive or deprecated operations from the OpenAPI document.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-sk-openapi-plugin-generator.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-openapi-plugin-generator.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Import third-party REST APIs dynamically into Semantic Kernel by feeding OpenAPI 3.0 specifications to runtime plugin generators.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-openapi-plugin-generator.**
- **Unmonitored runtime execution without telemetry in copilot-sk-openapi-plugin-generator.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-openapi-plugins"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
