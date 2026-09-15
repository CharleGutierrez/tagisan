---
name: agentic-baier-temporal-logic-ltl-ctl
description: "Linear Temporal Logic (LTL), Computation Tree Logic (CTL), Büchi automata, bisimulation equivalence, and probabilistic model checking."
triggers: ["baier", "katoen", "principles-model-checking", "ltl", "ctl", "buchi-automata", "bisimulation-equivalence"]
---

# agentic-baier-temporal-logic-ltl-ctl
> Based on **Principles of Model Checking - Christel Baier & Joost-Pieter Katoen**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **LTL Operators: Next (X), Globally/Always (G), Finally/Eventually (F), Until (U) over infinite execution paths.**
2. **Büchi Automata Translation: An LTL formula is converted into a non-deterministic Büchi automaton accepting infinite words that violate the property.**
3. **Bisimulation Equivalence: Systems S_1 and S_2 are bisimilar (S_1 ~ S_2) if they can simulate each other's transitions step-by-step.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Formulate critical async system invariants in LTL. Verify that every requested background task eventually terminates or reports an error.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Creating asynchronous event loops with no eventual termination or cancellation guarantee.**
- **Assuming path-based linear properties hold across branching computation trees without CTL checks.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "baier"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
