---
name: ba-cardinality-erd-relationship-rules
description: "Entity Relationship Modeling & Crow's Foot Notation: Relationship mandatory vs optional participation, foreign key constraints, and cascading integrity rules."
triggers: ["cardinality-erd-relationship-rules", "crows-foot", "richard-barker", "peter-chen", "cardinality-rules", "referential-integrity"]
---

# ba-cardinality-erd-relationship-rules
> Based on **CASE*Method: Entity Relationship Modelling - Peter Chen & Richard Barker**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Participation Invariant: Explicitly determine whether foreign keys are nullable (optional, 0..1) or NOT NULL (mandatory, 1..1).**
2. **Referential Integrity Cascades: Every foreign key must specify explicit `ON DELETE` behavior (`RESTRICT`, `CASCADE`, `SET NULL`).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Draw ER diagrams with strict Crow's Foot notation. Explicitly annotate nullability, unique constraints, and foreign key cascade behaviors.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Defaulting foreign keys to nullable without business justification.**
- **Omitting foreign key indexes, causing slow table-scan joins.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cardinality-erd-relationship-rules"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
