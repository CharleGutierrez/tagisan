---
name: agentic-martin-clean-architecture-boundaries
description: "Dependency Inversion Principle, concentric architectural boundaries, entities, use cases, interface adapters, and framework independence."
triggers: ["martin", "uncle-bob", "clean-architecture", "dependency-inversion", "concentric-boundaries", "use-cases", "framework-independence"]
---

# agentic-martin-clean-architecture-boundaries
> Based on **Clean Architecture: A Craftsman's Guide to Software Structure and Design - Robert C. Martin**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **The Dependency Rule: Source code dependencies must point only inward, toward higher-level policies: Entities -> Use Cases -> Adapters -> Frameworks.**
2. **Entities & Business Logic Purity: Enterprise business rules must have zero dependencies on databases, UI frameworks, or external third-party libraries.**
3. **Boundaries as Plugins: Databases and web delivery mechanisms are details that plug into the core application using interface ports.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce concentric boundaries in generated code: keep business entities strictly decoupled from database engines and web frameworks via ports and adapters.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Importing database ORM entities directly into domain logic or UI views.**
- **Letting external framework conventions dictate core business domain models.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "martin"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
