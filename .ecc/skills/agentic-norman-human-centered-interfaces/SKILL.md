---
name: agentic-norman-human-centered-interfaces
description: "Affordances, signifiers, conceptual models, feedback visibility, error tolerance, and bridging the gulf of execution and evaluation."
triggers: ["norman", "don-norman", "affordances-signifiers", "conceptual-models", "gulf-of-execution", "human-centered-design", "error-tolerance"]
---

# agentic-norman-human-centered-interfaces
> Based on **The Design of Everyday Things - Don Norman**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Gulf of Execution & Evaluation: Execution: how easily can the user figure out what to do; Evaluation: how easily can the user interpret system state.**
2. **Affordances & Signifiers: Affordances represent possible actions; signifiers communicate where and how action should take place.**
3. **Forcing Functions: Design constraints that make it physically or logically impossible to make destructive mistakes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design developer CLI and UI experiences with intuitive affordances and explicit signifiers. Implement confirmation forcing functions for destructive actions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Cryptic CLI tool flags with zero affordance or help feedback.**
- **Silent failures where operations complete with errors but return zero exit codes.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "norman"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
