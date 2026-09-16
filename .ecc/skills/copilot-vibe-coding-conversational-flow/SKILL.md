---
name: copilot-vibe-coding-conversational-flow
description: "Maintaining flow state in conversational development, conversational refactoring, and iterative steering."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "The Vibe Coding Paradigm: Conversational Software Construction - Andrej Karpathy"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["vibe-coding-flow", "conversational-programming", "iterative-steering", "flow-state-coding"]
---

# copilot-vibe-coding-conversational-flow
> Based on **The Vibe Coding Paradigm: Conversational Software Construction - Andrej Karpathy** (Cluster 9)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Conversational Invariant: Code via iterative English dialog, reviewing diffs and steering direction intuitively.**
2. **Epistemic Vigilance: Never trust generated code blindly; verify critical invariants and boundaries.**
3. **ALWAYS keep conversation context clean by resetting or summarizing historical turns.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-vibe-coding-conversational-flow.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-vibe-coding-conversational-flow actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Practice high-velocity vibe coding: see stuff, say stuff, run stuff, verify diffs, and steer the AI pair programmer.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-vibe-coding-conversational-flow.**
- **Unmonitored runtime execution without telemetry in copilot-vibe-coding-conversational-flow.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "vibe-coding-flow"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
