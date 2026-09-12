---
name: ba-state-machine-lifecycle-modeling
description: "Hierarchical State Machines (FSM & Statecharts): Orthogonal regions, guarded transitions, entry/exit actions, deterministic state lifecycles, and transition matrices."
triggers: ["state-machine-lifecycle-modeling", "state-machine", "statecharts", "david-harel", "fsm", "transition-matrix", "finite-state-machine"]
---

# ba-state-machine-lifecycle-modeling
> Based on **Statecharts: A Visual Formalism - David Harel & Martin Fowler**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **State Machine Determinism: For any state S and event E, at most one transition guard evaluates to true (deterministic next state).**
2. **Transition Completeness: The state transition matrix must explicitly define behavior for all (State x Event) combinations.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define entity lifecycles as a formal Finite State Machine (FSM): States, Events, Guards, Actions. Enforce strict transition checks in code.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mutating entity status without validating whether the current state permits the transition.**
- **Omitting error/cancelled states in entity lifecycles.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "state-machine-lifecycle-modeling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
