---
name: agentic-fowler-domain-specific-languages
description: "Internal vs external DSLs, semantic models, fluent interfaces, parser combinators, and language workbenches for business rule modeling."
triggers: ["fowler", "domain-specific-languages", "dsl-design", "fluent-interface", "semantic-model", "internal-dsl", "external-dsl"]
---

# agentic-fowler-domain-specific-languages
> Based on **Domain-Specific Languages - Martin Fowler**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Semantic Model Decoupling: The DSL syntax (internal builder or external script) populates a pure, syntax-agnostic semantic object graph.**
2. **Fluent Interface Protocol: Method chaining designed so sentences read as natural human language while remaining syntactically valid in host language.**
3. **Grammar-Driven External DSL: When business domain rules need to be edited by non-programmers without recompiling application binaries.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Create clean internal DSLs with fluent builders for complex configurations. Keep the underlying semantic model strictly decoupled from the syntax layer.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Coupling DSL parsing logic directly with execution side-effects instead of building a semantic model first.**
- **Creating clunky, unreadable method chaining that defeats the purpose of a fluent interface.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fowler"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
