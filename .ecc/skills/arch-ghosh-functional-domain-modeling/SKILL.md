---
name: arch-ghosh-functional-domain-modeling
description: "Functional domain architectures: Pure functional cores, Algebraic Data Types (ADTs), Monadic pipelines, reactive event streams, and side-effect isolation."
triggers: ["ghosh-functional", "functional-domain-modeling", "algebraic-data-types", "pure-core-imperative-shell", "monadic-error-handling", "reactive-streams"]
---

# arch-ghosh-functional-domain-modeling
> Based on **Functional and Reactive Domain Modeling - Debasish Ghosh**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Model domain entity state transitions as pure, deterministic functions: (State x Event) -> Result<NewState, DomainError>.**
2. **ALWAYS: Model all domain invariants as unrepresentable invalid states using Algebraic Data Types (Tagged Enums/Unions).**
3. **NEVER: Perform I/O, network calls, or database writes inside core domain entity methods.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Isolate all mutations to a pure functional core. Push all side-effects (database, network, file) to the outer imperative shell. Enforce exhaustive pattern matching on domain ADTs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Embedding database calls or clock lookups inside domain calculation methods.**
- **Using nullable primitive fields that permit illegal domain states.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-ghosh-functional-domain-modeling"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
