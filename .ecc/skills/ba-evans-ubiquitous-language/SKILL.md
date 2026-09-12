---
name: ba-evans-ubiquitous-language
description: "Ubiquitous Language & Strategic Modeling: Shared domain lexicon, eliminating translation layers, contextual isomorphism, and bounded linguistic contexts."
triggers: ["evans-ubiquitous-language", "ubiquitous-language", "eric-evans", "domain-driven-design", "ddd", "domain-lexicon"]
---

# ba-evans-ubiquitous-language
> Based on **Domain-Driven Design - Eric Evans**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Ubiquitous Language Invariant: If a term is not used by business domain experts in conversation, it must never appear as a class or table name.**
2. **Zero Synonym Drift: Prohibit using different terms for the same domain entity across files within the same bounded context.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain a strict GLOSSARY.md for the project. Use domain terms verbatim in code, class names, database tables, and API endpoints.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using technical jargon ('Record', 'DTO', 'Entity') in business conversation.**
- **Letting developers rename business concepts to suit technical habits.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "evans-ubiquitous-language"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
