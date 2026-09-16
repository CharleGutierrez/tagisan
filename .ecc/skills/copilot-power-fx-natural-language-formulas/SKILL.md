---
name: copilot-power-fx-natural-language-formulas
description: "Vibe coding Power Fx formulas from natural language, table filtering, and patch operations."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Power Fx: Low-Code Programming Language Guide - Greg Lindhorst"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-fx-vibe-coding", "natural-language-power-fx", "power-fx-patch", "power-fx-filtering"]
---

# copilot-power-fx-natural-language-formulas
> Based on **Power Fx: Low-Code Programming Language Guide - Greg Lindhorst** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Formula Generation: Translate natural language user requirements into declarative Power Fx statements.**
2. **Delegation Invariant: Ensure query formulas delegate execution to the underlying Dataverse/SQL server.**
3. **MANDATORY validation of delegation warnings to avoid client-side 500-record limits.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-power-fx-natural-language-formulas.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-power-fx-natural-language-formulas.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Generate and verify Power Fx formulas using natural language prompts, prioritizing delegable functions for large enterprise datasets.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-power-fx-natural-language-formulas.**
- **Unmonitored runtime execution without telemetry in copilot-power-fx-natural-language-formulas.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-fx-vibe-coding"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
