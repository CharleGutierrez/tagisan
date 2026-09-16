---
name: copilot-lost-in-the-middle-mitigation
description: "Structuring large context windows to counteract attentional decay in long-document reasoning."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Context Window Optimization & Token Budgeting - Greg Kamradt"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["lost-in-the-middle", "context-attentional-decay", "primacy-recency-structuring", "context-window-optimization"]
---

# copilot-lost-in-the-middle-mitigation
> Based on **Context Window Optimization & Token Budgeting - Greg Kamradt** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Attention Distribution: LLMs attend most strongly to tokens at the very beginning and very end of the prompt.**
2. **Placement Invariant: Place critical system instructions and final queries at the outer boundaries.**
3. **MANDATORY key fact repetition for context lengths exceeding 32,000 tokens.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-lost-in-the-middle-mitigation.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-lost-in-the-middle-mitigation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure large context windows strategically to counteract 'lost-in-the-middle' phenomena during complex document synthesis.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-lost-in-the-middle-mitigation.**
- **Unmonitored runtime execution without telemetry in copilot-lost-in-the-middle-mitigation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "lost-in-the-middle"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
