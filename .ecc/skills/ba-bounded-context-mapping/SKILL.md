---
name: ba-bounded-context-mapping
description: "Strategic Context Mapping: Bounded Context boundaries, Anti-Corruption Layers (ACL), Open Host Service (OHS), Shared Kernel, Customer-Supplier, and Conformist patterns."
triggers: ["bounded-context-mapping", "bounded-context", "context-mapping", "anti-corruption-layer", "acl", "vlad-khononov", "strategic-ddd"]
---

# ba-bounded-context-mapping
> Based on **Learning Domain-Driven Design - Vlad Khononov & Eric Evans**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Bounded Context Autonomy: A bounded context must own its data schema and be deployable independently of other contexts.**
2. **Anti-Corruption Layer (ACL): When integrating with legacy systems or third-party APIs, always insert an ACL to translate foreign data models into pure internal domain types.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Explicitly define Bounded Context boundaries and relationship patterns (ACL, Customer-Supplier, Conformist, Open Host Service) in CONTEXT_MAP.md.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Allowing multiple contexts to share read/write access to the same database tables.**
- **Letting upstream vendor schemas leak directly into internal domain models.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bounded-context-mapping"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
