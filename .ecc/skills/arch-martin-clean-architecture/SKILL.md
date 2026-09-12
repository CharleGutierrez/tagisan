---
name: arch-martin-clean-architecture
description: "Hexagonal and Clean Architecture: Dependency Inversion Principle, Boundary Crossings, Entities, Use Cases, Interface Adapters, and Framework Independence."
triggers: ["clean-architecture", "uncle-bob", "hexagonal-architecture", "ports-and-adapters", "dependency-inversion", "onion-architecture", "use-case-interactor"]
---

# arch-martin-clean-architecture
> Based on **Clean Architecture: A Craftsman's Guide to Software Structure - Robert C. Martin**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Source code dependencies must point strictly inwards: inner business use cases know nothing about outer frameworks, databases, or UI.**
2. **ALWAYS: Outer infrastructure layers implement interfaces defined by the inner domain (Dependency Inversion Principle).**
3. **NEVER: Import web frameworks (e.g. Express, Axum, Actix, Spring) or ORM models inside domain entities.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure codebases into Entities, Use Cases, Interface Adapters, and Frameworks. Expose domain capabilities via Ports and plug in external dependencies via Adapters.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Importing database connection pools directly into domain entity files.**
- **Coupling business validation rules to HTTP request parameters.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-martin-clean-architecture"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
