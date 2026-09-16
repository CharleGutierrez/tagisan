---
name: copilot-studio-power-fx-formulas
description: "Utilizing Power Fx for variable transformations, regex parsing, and condition evaluation in topics."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Power Fx: Low-Code Programming Language Guide - Greg Lindhorst"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-fx-formulas", "power-fx-copilot-studio", "variable-transformations", "power-fx-conditions"]
---

# copilot-studio-power-fx-formulas
> Based on **Power Fx: Low-Code Programming Language Guide - Greg Lindhorst** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Declarative Language: Power Fx expressions evaluate synchronously within topic nodes.**
2. **Type Safety: Strong typing across Text, Number, Boolean, Record, and Table types.**
3. **ALWAYS handle null/blank values explicitly using `IsBlank()` or `Coalesce()`.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-power-fx-formulas.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-studio-power-fx-formulas actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Write clean, declarative Power Fx expressions in Set Variable and Condition nodes for deterministic data wrangling.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Complex procedural logic written in massive nested Power Fx formulas instead of delegating to Power Automate.**
- **Unchecked string manipulation leading to runtime type errors on null inputs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-fx-formulas"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
