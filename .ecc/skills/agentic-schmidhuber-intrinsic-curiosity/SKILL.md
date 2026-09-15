---
name: agentic-schmidhuber-intrinsic-curiosity
description: "Compression progress as intrinsic reward, artificial curiosity, Gödel machines, self-invented problems, and mathematically optimal self-improvement."
triggers: ["schmidhuber", "intrinsic-curiosity", "compression-progress", "godel-machine", "self-improving-agents", "formal-theory-fun"]
---

# agentic-schmidhuber-intrinsic-curiosity
> Based on **Formal Theory of Fun & Intrinsic Motivation in Self-Improving Agents - Jürgen Schmidhuber**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Compression Progress Reward: Intrinsic reward R_intrinsic(t) = C(state | model_{t-1}) - C(state | model_t), rewarding the discovery of compressible regularities.**
2. **Gödel Machine Invariant: A self-referential system that rewires its own code if and only if it can formally prove the modification yields superior expected utility.**
3. **Artificial Curiosity: Actively seeking out environments where the agent's current predictive model makes errors, accelerating learning.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Incentivize agents to proactively explore and refactor confusing, high-entropy legacy code modules. Reward the discovery of simplifying abstractions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Allowing self-modifying agents to alter core safety invariants without mathematical proof of safety.**
- **Focusing exclusively on easy, familiar code while avoiding poorly understood mission-critical modules.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "schmidhuber"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
