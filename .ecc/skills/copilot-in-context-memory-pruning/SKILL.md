---
name: copilot-in-context-memory-pruning
description: "Sliding window attention, context summarization, and key-value memory compression."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Context Window Optimization & Token Budgeting - Greg Kamradt"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["memory-pruning", "sliding-window-context", "context-summarization", "kv-memory-compression"]
---

# copilot-in-context-memory-pruning
> Based on **Context Window Optimization & Token Budgeting - Greg Kamradt** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Pruning Strategy: Evict older conversation turns when token consumption exceeds 75% of context window.**
2. **Rolling Summary: Compress evicted turns into a concise running executive summary stored in system state.**
3. **ALWAYS preserve core system instructions and initial user constraints.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-in-context-memory-pruning.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-in-context-memory-pruning actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement dynamic memory pruning and rolling summarization to sustain coherent multi-hour Copilot conversation sessions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-in-context-memory-pruning.**
- **Unmonitored runtime execution without telemetry in copilot-in-context-memory-pruning.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "memory-pruning"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
