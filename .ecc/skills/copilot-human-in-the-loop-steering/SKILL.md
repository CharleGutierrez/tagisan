---
name: copilot-human-in-the-loop-steering
description: "Designing graceful human intervention points, interruptible agent plans, and approval checkpoints."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Co-Intelligence: Living and Working with AI - Ethan Mollick"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["human-in-the-loop", "agent-steering-points", "approval-checkpoints", "interruptible-plans"]
---

# copilot-human-in-the-loop-steering
> Based on **Co-Intelligence: Living and Working with AI - Ethan Mollick** (Cluster 9)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Steering Invariant: High-stakes mutations (sending emails, modifying permissions) MUST require human sign-off.**
2. **State Resumption: Agents must pause gracefully awaiting approval and resume from saved checkpoints.**
3. **NEVER allow autonomous execution of financial or destructive actions without human confirmation.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-human-in-the-loop-steering.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-human-in-the-loop-steering actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Architect human-in-the-loop approval gates inside Copilot agents using Teams approval cards and pause/resume states.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-human-in-the-loop-steering.**
- **Unmonitored runtime execution without telemetry in copilot-human-in-the-loop-steering.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "human-in-the-loop"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
