---
name: copilot-studio-custom-topic-state-machine
description: "Classic topic dialog management, conditional branching, slot filling, and redirect nodes."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Mastering Microsoft Copilot Studio - Robert Kaack"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["custom-topic-state-machine", "dialog-management", "conditional-branching", "slot-filling"]
---

# copilot-studio-custom-topic-state-machine
> Based on **Mastering Microsoft Copilot Studio - Robert Kaack** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Deterministic State: Deterministic nodes (Question, Message, Condition, Action) executed sequentially.**
2. **Variable Scoping: Variables scoped to Topic, Global, or System levels.**
3. **MANDATORY redirect to a Fallback topic when input cannot be parsed after N retries.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-studio-custom-topic-state-machine.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-custom-topic-state-machine.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use classic topics for strict transactional flows (e.g., password reset, fund transfers, formal approvals) where generative deviations are prohibited.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Deeply nested conditional trees (>5 levels) that become impossible to debug.**
- **Infinite loops between topic redirects with no exit condition.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "custom-topic-state-machine"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
