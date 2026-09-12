---
name: ba-requirements-traceability-matrix
description: "Bidirectional Requirements Traceability Matrix (RTM): Forward and backward traceability, gap analysis, orphan detection, and verification coverage."
triggers: ["requirements-traceability-matrix", "rtm", "bidirectional-traceability", "orphan-detection", "ieee-29148", "compliance-matrix"]
---

# ba-requirements-traceability-matrix
> Based on **IEEE Std 830 / ISO/IEC/IEEE 29148 - Requirements Engineering Standards**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Bidirectional Linkage: Business Need <-> System Requirement <-> Architecture Component <-> Source Code <-> Automated Test Case.**
2. **Zero Orphan Rule: Every line of application code and every test must trace back to an authorized requirement.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain a bidirectional RTM. Flag any requirement with 0 test cases as an unverified defect. Flag any code feature with 0 requirements as unauthorized scope creep.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Building features that have no upstream business justification.**
- **Writing unit tests that test implementation details instead of requirement criteria.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "requirements-traceability-matrix"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
