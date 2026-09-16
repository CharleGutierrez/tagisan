---
name: copilot-studio-bot-framework-composer-interop
description: "Integrating Bot Framework Composer dialogs and Bot Framework SDK components."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Programming the Microsoft Bot Framework - Joe Mayo"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["bot-framework-composer", "composer-interop", "bot-framework-sdk", "adaptive-dialogs"]
---

# copilot-studio-bot-framework-composer-interop
> Based on **Programming the Microsoft Bot Framework - Joe Mayo** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Adaptive Dialogs: Declarative JSON-based dialog trees exported from Composer into Copilot Studio.**
2. **Custom Code Components: Integrating Azure Functions and Bot Framework middleware.**
3. **ALWAYS maintain backward compatibility of schema properties when modifying dialog definitions.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-bot-framework-composer-interop.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-studio-bot-framework-composer-interop actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Extend Copilot Studio with Bot Framework Composer for advanced card interactions, complex regex recognizers, and custom telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Directly editing raw Composer JSON without testing in Bot Framework Emulator.**
- **Introducing unmanaged state variables that conflict with Copilot Studio system variables.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bot-framework-composer"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
