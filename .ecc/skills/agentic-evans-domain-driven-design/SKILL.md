---
name: agentic-evans-domain-driven-design
description: "Ubiquitous Language, Bounded Contexts, Entities, Value Objects, Aggregates, Repositories, Domain Services, and Anti-Corruption Layers."
triggers: ["evans", "domain-driven-design", "ddd", "ubiquitous-language", "bounded-context", "aggregates", "anti-corruption-layer"]
---

# agentic-evans-domain-driven-design
> Based on **Domain-Driven Design: Tackling Complexity in the Heart of Software - Eric Evans**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Ubiquitous Language: A common, rigorous language shared by developers and domain experts, reflected directly in the source code.**
2. **Bounded Context: A clear linguistic and architectural boundary within which a specific domain model applies and remains internally consistent.**
3. **Aggregate Root Invariant: Aggregates are clusters of associated objects treated as a unit for data changes; all external access must go through the Root.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model software domains around strict Aggregate boundaries. Enforce Ubiquitous Language consistently in types, variable names, and database schemas.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Anemic domain models where entities are dumb data bags manipulated by bloated procedural services.**
- **Leaking model concepts across Bounded Contexts without an Anti-Corruption Layer.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "evans"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
