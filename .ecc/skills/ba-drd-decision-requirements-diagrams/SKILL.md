---
name: ba-drd-decision-requirements-diagrams
description: "Decision Requirements Diagrams (DRD): Decision nodes, Input Data nodes, Business Knowledge Models (BKMs), Knowledge Sources, and decision decomposition."
triggers: ["drd-decision-requirements-diagrams", "drd", "decision-requirements-diagram", "bkm", "business-knowledge-models", "dmn-decomposition"]
---

# ba-drd-decision-requirements-diagrams
> Based on **Decision Model and Notation (DMN 1.5) - OMG Standard**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **DRD Hierarchy: High-level decisions decompose into sub-decisions, input data, and reusable Business Knowledge Models (BKMs).**
2. **Process-Decision Separation: BPMN process tasks invoke DMN decision services via clean input/output interfaces.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model complex decision systems as a DRD: Link Input Data -> Sub-decisions -> Final Decision, externalizing calculations into BKMs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing monolithic 500-line decision scripts without decomposing into sub-decisions.**
- **Mixing procedural workflow routing with business calculation logic.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "drd-decision-requirements-diagrams"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
