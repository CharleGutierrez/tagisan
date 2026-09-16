---
name: copilot-teams-ai-turn-state-management
description: "ConversationState, UserState, and TempState scoping, validation, and storage providers (Cosmos/Memory)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Conversation and User State Management in Teams Bots - Joe Stagner"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["turn-state-management", "teams-ai-state", "conversationstate-userstate", "cosmos-state-storage"]
---

# copilot-teams-ai-turn-state-management
> Based on **Conversation and User State Management in Teams Bots - Joe Stagner** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **State Scopes: `conversation` (persisted across users in chat), `user` (persisted per user), `temp` (single turn).**
2. **Storage Invariant: NEVER use in-memory storage in multi-instance production environments; use Cosmos DB or Blob.**
3. **ALWAYS initialize state with safe default factory functions.**
4. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-ai-turn-state-management actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Manage agent memory by partitioning data cleanly across conversation, user, and temporary turn state scopes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-turn-state-management.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-turn-state-management.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "turn-state-management"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
