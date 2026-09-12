---
name: ba-hoberman-data-modeling-resource
description: "Conceptual, Logical & Physical Data Modeling: Entity definitions, cardinalities, relational boundaries, data dictionaries, and ERD verification."
triggers: ["hoberman-data-modeling-resource", "steve-hoberman", "data-modeling", "conceptual-model", "logical-data-model", "erd-modeling"]
---

# ba-hoberman-data-modeling-resource
> Based on **Data Modeling Made Simple (2nd Edition) - Steve Hoberman**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Three-Level Schema Architecture: Conceptual (Business entities) -> Logical (Attributes, normalized, keys) -> Physical (Data types, indexes, partitions).**
2. **Cardinality Invariant: Every relationship between Entity A and Entity B must specify minimum and maximum cardinality (0..1, 1..1, 0..N, 1..N) on both ends.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model data at Conceptual, Logical, and Physical levels. Document all cardinalities (1:1, 1:N, M:N) and foreign key constraints in ERD diagrams.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Generating physical tables without first modeling conceptual entities and cardinalities.**
- **Using many-to-many relationships without an explicit junction table.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "hoberman-data-modeling-resource"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
