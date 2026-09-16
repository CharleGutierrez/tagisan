---
name: copilot-excel-javascript-api-custom-functions
description: "Excel JavaScript API custom functions (=TGS.*), streaming functions, and matrix calculation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Developing Office Add-ins with Office.js - Michael Saunders"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["excel-custom-functions", "office-js-functions", "streaming-custom-functions", "excel-matrix-calculation"]
---

# copilot-excel-javascript-api-custom-functions
> Based on **Developing Office Add-ins with Office.js - Michael Saunders** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Custom Function Contract: Annotate functions with JSDoc `@customfunction` tags and unique function names.**
2. **Streaming Invariant: Streaming functions emit continuous values via `CustomFunctions.StreamingInvocation`.**
3. **ALWAYS handle calculation cancellation via the `invocation.onCanceled` event.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-excel-javascript-api-custom-functions.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-excel-javascript-api-custom-functions actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Develop custom Excel calculation formulas in TypeScript that query AI models or external data sources asynchronously.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-excel-javascript-api-custom-functions.**
- **Unmonitored runtime execution without telemetry in copilot-excel-javascript-api-custom-functions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "excel-custom-functions"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
