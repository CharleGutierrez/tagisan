---
name: agentic-pierce-type-systems-soundness
description: "Simply typed lambda calculus, type safety (progress and preservation theorems), subtyping, parametric polymorphism, and Curry-Howard isomorphism."
triggers: ["pierce", "tapl", "type-systems", "type-soundness", "progress-preservation", "lambda-calculus", "curry-howard"]
---

# agentic-pierce-type-systems-soundness
> Based on **Types and Programming Languages (TAPL) - Benjamin C. Pierce**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Type Safety = Progress + Preservation: Progress: A well-typed term is either a value or can take an evaluation step. Preservation: If t : T and t -> t', then t' : T.**
2. **Curry-Howard Isomorphism: Types correspond to logical propositions; programs correspond to proofs of those propositions.**
3. **Subtyping Invariant (Liskov): S <: T means any term of type S can be safely used in a context expecting type T.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Leverage rich static type systems (Rust, TypeScript) to encode business invariants into types. Make illegal states unrepresentable at compile time.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using stringly-typed or unstructured `any` types that bypass compiler safety verification.**
- **Violating the preservation theorem by writing unsafe casts that cause runtime type crashes.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "pierce"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
