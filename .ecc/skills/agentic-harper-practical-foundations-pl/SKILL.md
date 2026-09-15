---
name: agentic-harper-practical-foundations-pl
description: "Abstract binding trees, structural operational semantics, inductive definitions, dynamic dispatch vs static typing, and language modularity."
triggers: ["harper", "pfpl", "operational-semantics", "abstract-binding-trees", "inductive-definitions", "type-theory"]
---

# agentic-harper-practical-foundations-pl
> Based on **Practical Foundations for Programming Languages - Robert Harper**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Structural Operational Semantics (SOS): Defining computation steps via inductive inference rules over abstract syntax terms.**
2. **Abstract Binding Trees (ABTs): Enriching ASTs with formal variable binding, alpha-equivalence, and capture-avoiding substitution.**
3. **Static/Dynamic Phase Distinction: Strict separation between compile-time static analysis and runtime dynamic evaluation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define language extensions and domain primitives using rigorous operational semantics. Enforce capture-avoiding substitution in code generation templates.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Naive macro expansions that cause variable name collisions (accidental variable capture).**
- **Blurring the phase distinction by executing dynamic runtime logic during static build steps.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "harper"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
