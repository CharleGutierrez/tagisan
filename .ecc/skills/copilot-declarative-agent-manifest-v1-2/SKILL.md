---
name: copilot-declarative-agent-manifest-v1-2
description: "Declarative agent manifest schema v1.2 (declarativeAgent.json), instructions, capabilities, and actions."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft 365 Declarative Agent Specification - Microsoft Learn"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["declarative-agent-manifest", "declarativeagent-json", "declarative-manifest-v1-2", "agent-schema"]
---

# copilot-declarative-agent-manifest-v1-2
> Based on **Microsoft 365 Declarative Agent Specification - Microsoft Learn** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Schema Conformance: `$schema: https://developer.microsoft.com/json-schemas/copilot/declarative-agent/v1.2/schema.json`.**
2. **Instruction Architecture: System instructions must define Identity, Scope, Constraints, Response Format, and Tone.**
3. **Capabilities Array: Explicit declaration of `OneDriveAndSharePoint`, `GraphConnectors`, and `WebSearch`.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-declarative-agent-manifest-v1-2.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-declarative-agent-manifest-v1-2.**
6. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-declarative-agent-manifest-v1-2 actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Author declarativeAgent.json with rigorous type declarations. ALWAYS reference valid external action plugin files via relative paths in `actions`.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Including executable scripts or unescaped control characters in system instructions.**
- **Referencing non-existent action plugin IDs in the manifest capabilities array.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "declarative-agent-manifest"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
