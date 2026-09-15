---
name: agentic-anderson-actr-cognitive-memory
description: "Declarative vs procedural memory, chunk activation equation, base-level learning, production rules, pattern matching, and cognitive memory retrieval."
triggers: ["anderson", "act-r", "declarative-procedural-memory", "chunk-activation", "base-level-learning", "cognitive-architecture-memory"]
---

# agentic-anderson-actr-cognitive-memory
> Based on **The Architecture of Cognition (ACT-R Memory) - John R. Anderson**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Base-Level Activation Equation: A_i = B_i + sum_j W_j * S_{ji} + epsilon, where B_i = ln(sum_{k=1}^n t_k^{-d}) decays as a power law of time.**
2. **Declarative Chunks vs Production Rules: Facts/schemas reside in declarative memory; executable cognitive actions reside in procedural IF-THEN rules.**
3. **Conflict Resolution: Selecting the production rule with the highest expected utility when multiple rules match the current goal buffer.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model long-term agent memory using ACT-R activation equations. Decay historical conversation turns while boosting recently and frequently accessed code modules.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Treating all historical memories as equally relevant regardless of age or frequency of use.**
- **Mixing procedural execution logic with static declarative schemas.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "anderson"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
