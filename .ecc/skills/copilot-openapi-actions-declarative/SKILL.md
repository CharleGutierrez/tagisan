---
name: copilot-openapi-actions-declarative
description: "Authoring OpenAPI specs for Declarative Agent actions, JSON schema parameters, and output extraction rules."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["openapi-actions", "declarative-agent-actions", "action-openapi-spec", "copilot-action-plugin"]
---

# copilot-openapi-actions-declarative
> Based on **RESTful API Design & OpenAPI 3.0 for AI Agents - Arnaud Lauret** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **OpenAPI 3.0.x Standard: Strict adherence to JSON/YAML OpenAPI specification.**
2. **OperationId Invariant: Every path operation MUST define a unique, human-readable `operationId`.**
3. **Parameter Grounding: All path, query, and body parameters must include explicit types and descriptive summaries.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-openapi-actions-declarative.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-openapi-actions-declarative.**
6. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-openapi-actions-declarative actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design action plugin specs with minimal endpoint payloads. Filter out unused enterprise endpoints to keep the agent toolset crisp.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Exposing endpoints returning multi-megabyte unstructured JSON arrays that exceed model context limits.**
- **Missing `operationId` leading to orchestrator tool invocation failure.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "openapi-actions"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
