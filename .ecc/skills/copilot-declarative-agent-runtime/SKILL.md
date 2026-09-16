---
name: copilot-declarative-agent-runtime
description: "Declarative Agent sandboxed runtime execution loop, capability dispatching, and prompt boundary confinement."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft 365 Declarative Agent Specification - Microsoft Learn"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["declarative-agent-runtime", "sandboxed-execution", "capability-dispatching", "prompt-boundary-confinement"]
---

# copilot-declarative-agent-runtime
> Based on **Microsoft 365 Declarative Agent Specification - Microsoft Learn** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Sandbox Runtime: Declarative agents execute strictly within the M365 Copilot conversation shell without arbitrary custom code execution.**
2. **Capability Scoping: Actions are bounded by explicit OpenAPI 3.0 operation definitions and Graph connection declarations.**
3. **MANDATORY validation of action payloads against declared JSON schemas prior to execution.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-declarative-agent-runtime.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-declarative-agent-runtime.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define Declarative Agents with precise system instructions, explicit capability scopes (OneDrive, SharePoint, Web, GraphConnectors), and typed Action Plugins.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Over-parameterizing instructions with contradictory persona directives.**
- **Omitting response schema definitions in action specifications, leading to unstructured output hallucination.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "declarative-agent-runtime"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
