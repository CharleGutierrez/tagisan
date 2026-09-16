---
name: copilot-adaptive-cards-v1-5-templating
description: "Adaptive Cards v1.5 schema, Adaptive Card Templating (AdaptiveCardTemplate), and data binding."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["adaptive-cards-templating", "adaptive-cards-v1-5", "card-data-binding", "adaptivecardtemplate"]
---

# copilot-adaptive-cards-v1-5-templating
> Based on **Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Separation of Concerns: Separate static card JSON template from dynamic runtime `$data` context.**
2. **Schema Compliance: Target schema version 1.5 for maximum Teams and Outlook desktop/mobile compatibility.**
3. **ALWAYS validate template syntax using official `adaptivecards-templating` library.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-adaptive-cards-v1-5-templating.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-adaptive-cards-v1-5-templating actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Author elegant, decoupled Adaptive Cards using templating engines and dynamic JSON data payloads.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-adaptive-cards-v1-5-templating.**
- **Unmonitored runtime execution without telemetry in copilot-adaptive-cards-v1-5-templating.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "adaptive-cards-templating"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
