---
name: copilot-universal-actions-for-teams
description: "Universal Action Model: Action.Execute, refresh triggers, and user-specific card views."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Universal Actions in Adaptive Cards: Action.Execute - Teams Engineering"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["universal-actions-teams", "action-execute", "user-specific-views", "card-refresh-triggers"]
---

# copilot-universal-actions-for-teams
> Based on **Universal Actions in Adaptive Cards: Action.Execute - Teams Engineering** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Action.Execute Model: Replace legacy `Action.Submit` and `Action.Http` with unified `Action.Execute`.**
2. **User-Specific Views: Return customized card views tailored to the specific user viewing the card in group chats.**
3. **MANDATORY verification of `context.action.verb` and payload data.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-universal-actions-for-teams.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-universal-actions-for-teams.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Universal Action Model (`Action.Execute`) in Teams cards, delivering dynamic user-specific views and live refreshes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-universal-actions-for-teams.**
- **Unmonitored runtime execution without telemetry in copilot-universal-actions-for-teams.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "universal-actions-teams"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
