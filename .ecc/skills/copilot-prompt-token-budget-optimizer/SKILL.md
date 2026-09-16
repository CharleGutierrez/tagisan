---
name: copilot-prompt-token-budget-optimizer
description: "Calculating prompt and completion token budgets, parameter tradeoffs, and cost modeling."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Context Window Optimization & Token Budgeting - Greg Kamradt"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["token-budget-optimizer", "context-budgeting", "cost-modeling-tokens", "prompt-token-limits"]
---

# copilot-prompt-token-budget-optimizer
> Based on **Context Window Optimization & Token Budgeting - Greg Kamradt** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Budget Allocation: Fixed allocation for System (15%), Knowledge Context (60%), History (15%), Output (10%).**
2. **Hard Token Counter: Calculate exact token counts using BPE tokenizers (tiktoken) before calling inference APIs.**
3. **MANDATORY token truncation guards preventing unexpected 400 Bad Request overflow errors.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-prompt-token-budget-optimizer.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-prompt-token-budget-optimizer.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Manage prompt token budgets algorithmically, optimizing context allocation to maximize accuracy while minimizing enterprise token costs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-prompt-token-budget-optimizer.**
- **Unmonitored runtime execution without telemetry in copilot-prompt-token-budget-optimizer.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "token-budget-optimizer"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
