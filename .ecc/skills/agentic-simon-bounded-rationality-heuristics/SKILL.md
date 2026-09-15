---
name: agentic-simon-bounded-rationality-heuristics
description: "Bounded rationality, satisficing vs optimizing, near-decomposability of complex systems, and cognitive architecture heuristics."
triggers: ["simon", "herbert-simon", "bounded-rationality", "satisficing", "near-decomposability", "sciences-of-the-artificial"]
---

# agentic-simon-bounded-rationality-heuristics
> Based on **The Sciences of the Artificial (3rd ed) - Herbert A. Simon**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Bounded Rationality: Decision-makers lack the cognitive resources and complete information required to find mathematically optimal solutions.**
2. **Satisficing Principle: Select the first candidate solution that meets or exceeds predefined aspiration thresholds rather than searching for the global optimum.**
3. **Near-Decomposability: Complex systems can be decomposed into subsystems whose internal interactions are much stronger than interactions between subsystems.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Apply satisficing criteria to code synthesis: accept solutions that pass all tests and meet performance budgets without endlessly striving for theoretical perfection.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Paralysis by analysis: searching indefinitely for the 'perfect' algorithm when a standard solution is 100% adequate.**
- **Monolithic coupling that destroys the near-decomposability of software modules.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "simon"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
