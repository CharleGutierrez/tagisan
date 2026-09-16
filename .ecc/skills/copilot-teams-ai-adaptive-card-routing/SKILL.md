---
name: copilot-teams-ai-adaptive-card-routing
description: "app.adaptiveCards.actionSubmit handlers, universal actions, and form data validation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["adaptive-card-routing", "teams-ai-card-routing", "action-submit-handler", "card-data-validation"]
---

# copilot-teams-ai-adaptive-card-routing
> Based on **Mastering Adaptive Cards: Interactive UI for Microsoft 365 - Matt Hidinger** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Action Routing: Route card submissions via `app.adaptiveCards.actionSubmit(verb, handler)`.**
2. **Data Sanitization: Strictly validate form inputs before invoking database mutations.**
3. **ALWAYS acknowledge card submissions within 10 seconds to avoid Teams client timeout.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-ai-adaptive-card-routing.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-ai-adaptive-card-routing actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Handle interactive Adaptive Card submissions using dedicated action handlers, returning updated card views or confirmation messages.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-adaptive-card-routing.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-adaptive-card-routing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "adaptive-card-routing"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
