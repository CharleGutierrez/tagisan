---
name: copilot-interactive-cards-live-updates
description: "Updating posted messages in Teams channels and chats in-place with UpdateCard operations."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Universal Actions in Adaptive Cards: Action.Execute - Teams Engineering"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["interactive-cards-live-updates", "in-place-card-update", "update-card-teams", "live-card-refreshes"]
---

# copilot-interactive-cards-live-updates
> Based on **Universal Actions in Adaptive Cards: Action.Execute - Teams Engineering** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **In-Place Refresh: Update existing message card using `TurnContext.updateActivity()` with identical activity ID.**
2. **Concurrency Safety: Avoid race conditions when multiple users click card buttons simultaneously.**
3. **ALWAYS show updated status indicators (e.g. 'Approved by Alice at 14:02').**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-interactive-cards-live-updates.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-interactive-cards-live-updates actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Update posted Teams cards dynamically in-place to display workflow progression, approvals, and real-time statuses.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-interactive-cards-live-updates.**
- **Unmonitored runtime execution without telemetry in copilot-interactive-cards-live-updates.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "interactive-cards-live-updates"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
