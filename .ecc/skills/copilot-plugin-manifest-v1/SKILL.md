---
name: copilot-plugin-manifest-v1
description: "Microsoft Copilot plugin schema v1.0/v1.1, OpenAPI 3.0 specification binding, and ai-plugin.json contracts."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["plugin-manifest", "ai-plugin-json", "openapi-copilot-binding", "plugin-schema-v1"]
---

# copilot-plugin-manifest-v1
> Based on **RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Manifest Binding: `ai-plugin.json` specifies human and model descriptions, auth schemes, and relative OpenAPI endpoints.**
2. **Description Informativeness Invariant: Function and parameter descriptions MUST explain 'what', 'why', and 'format' to enable zero-shot tool selection.**
3. **ALWAYS specify authentication flows (Anonymous, OAuth2, ApiKey) explicitly in the securityScheme definition.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-plugin-manifest-v1.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-plugin-manifest-v1 actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Author rigorous OpenAPI 3.0 specs with crisp semantic descriptions for all paths and parameters. Validate manifests against the official Microsoft Copilot plugin schema.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Vague parameter descriptions (e.g., 'id: string') that cause the model to guess argument formats.**
- **Exposing state-mutating POST/DELETE operations without requiring user confirmation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "plugin-manifest"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
