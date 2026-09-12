---
name: ba-relational-normalization-3nf
description: "Relational Normalization & Normal Forms: 1NF atomicity, 2NF partial key dependency elimination, 3NF transitive dependency elimination, and Boyce-Codd Normal Form (BCNF)."
triggers: ["relational-normalization-3nf", "normalization", "3nf", "bcnf", "codd-date", "functional-dependencies", "database-normalization"]
---

# ba-relational-normalization-3nf
> Based on **An Introduction to Database Systems - E.F. Codd & C.J. Date**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **First Normal Form (1NF): All attributes must be atomic; no repeating groups or serialized arrays in single columns.**
2. **Second Normal Form (2NF): In 1NF and every non-key attribute is fully functionally dependent on the entire primary key.**
3. **Third Normal Form (3NF): In 2NF and no non-key attribute is transitively dependent on the primary key.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Verify all relational tables comply with 3NF/BCNF. Decompose partial and transitive dependencies into normalized child tables.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Storing comma-separated lists or unstructured JSON in relational columns requiring query filtering.**
- **Premature denormalization before establishing baseline 3NF.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "relational-normalization-3nf"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
