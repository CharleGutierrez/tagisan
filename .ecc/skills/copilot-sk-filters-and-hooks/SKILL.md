---
name: copilot-sk-filters-and-hooks
description: "Function invocation filters, prompt render filters, and authorization middleware in Semantic Kernel."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Semantic Kernel Filters, Hooks and Observability - Microsoft AI"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-filters-hooks", "invocation-filters", "prompt-render-filters", "sk-middleware"]
---

# copilot-sk-filters-and-hooks
> Based on **Semantic Kernel Filters, Hooks and Observability - Microsoft AI** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Filter Types: `IFunctionInvocationFilter`, `IPromptRenderFilter`, `IAutoFunctionInvocationFilter`.**
2. **Short-Circuit Capability: Filters can inspect arguments, modify outputs, or abort execution prior to invocation.**
3. **ALWAYS log execution duration and token counts inside filter hooks.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-filters-and-hooks.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-sk-filters-and-hooks actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement cross-cutting concerns (security auditing, secret scrubbing, caching, telemetry) using Semantic Kernel filter pipelines.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-filters-and-hooks.**
- **Unmonitored runtime execution without telemetry in copilot-sk-filters-and-hooks.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-filters-hooks"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
