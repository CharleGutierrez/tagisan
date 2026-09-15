---
name: agentic-clarke-model-checking-invariants
description: "State transition systems, temporal logic formulas, safety and liveness properties, state-space explosion mitigation, Binary Decision Diagrams (BDD)."
triggers: ["clarke", "grumberg", "peled", "model-checking", "temporal-logic", "safety-liveness", "state-space-verification"]
---

# agentic-clarke-model-checking-invariants
> Based on **Model Checking - Edmund M. Clarke, Orna Grumberg & Doron A. Peled**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Safety vs Liveness: Safety: 'Bad things never happen' (G ~bad). Liveness: 'Good things eventually happen' (F good).**
2. **Kripke Structure Formalization: M = (S, S_0, R, L), verifying whether M satisfies temporal formula phi (M |= phi).**
3. **Counterexample Generation: Model checkers provide exact execution traces demonstrating how an invariant is violated.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Verify critical concurrency locks, consensus protocols, and state machines against formal safety and liveness invariants.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Confusing safety with liveness, ignoring deadlock states where no bad state occurs but progress ceases.**
- **State explosions caused by modeling unconstrained integers instead of bounded abstractions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "clarke"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
