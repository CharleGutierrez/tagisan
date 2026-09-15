---
name: agentic-gulwani-program-synthesis
description: "Inductive program synthesis, programming by example (PBE), domain-specific Version Space Algebras, syntax-guided synthesis (SyGuS), and deductive search."
triggers: ["gulwani", "polozov", "singh", "program-synthesis", "programming-by-example", "version-space-algebra", "sygus"]
---

# agentic-gulwani-program-synthesis
> Based on **Program Synthesis - Sumit Gulwani, Oleksandr Polozov & Rishabh Singh**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Inductive Synthesis Invariant: Given input-output examples {(x_1, y_1), ..., (x_n, y_n)}, synthesize program P in DSL such that forall i, P(x_i) == y_i.**
2. **Version Space Algebra (VSA): Compactly represent an exponential number of consistent candidate programs using a polynomial-sized shared DAG.**
3. **Deductive Top-Down Search: Propagate input-output constraints downward through grammar operators to prune invalid program spaces early.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Synthesize data transformation pipelines and regex extractors using programming-by-example principles. Verify candidate programs against test suites before proposing them.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Proposing code without checking that it passes the user's provided input-output examples.**
- **Generating overly complex general programs when a simple DSL expression satisfies all constraints.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "gulwani"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
