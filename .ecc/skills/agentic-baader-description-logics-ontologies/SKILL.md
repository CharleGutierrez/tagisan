---
name: agentic-baader-description-logics-ontologies
description: "Description Logics (ALC, SHOIN), TBox (terminological) vs ABox (assertional) reasoning, tableau algorithms, and ontology subsumption."
triggers: ["baader", "description-logics", "tbox-abox", "tableau-algorithm", "ontology-subsumption", "formal-knowledge-base"]
---

# agentic-baader-description-logics-ontologies
> Based on **The Description Logic Handbook: Theory, Implementation, and Applications - Franz Baader et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **TBox vs ABox: TBox defines conceptual schema axioms (e.g. 'AdminUser subclass of User'); ABox defines concrete instance assertions (e.g. 'alice instance of AdminUser').**
2. **Subsumption Checking: Determining if concept C is subsumed by concept D (C sqsubseteq D) under all valid interpretations.**
3. **Tableau Decidability: Applying tableau expansion rules to systematically verify ontology satisfiability and consistency.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Verify that agent-generated architectural schemas and access control models are logically consistent using description logic subsumption checkers.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Defining cyclical ontology hierarchies with unsatisfiable concept definitions.**
- **Confusing class-level schema modifications (TBox) with instance-level data modifications (ABox).**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "baader"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
