---
name: ba-wiegers-requirements-engineering
description: "Three-tier requirements engineering: Business Requirements (Vision & Scope), User Requirements (Tasks & Use Cases), and Functional Requirements (Invariants, RTM, Planguage quality attributes)."
triggers: ["wiegers-requirements-engineering", "wiegers", "software-requirements", "requirements-traceability", "planguage", "functional-requirements", "requirements-engineering", "prd-specification"]
---

# ba-wiegers-requirements-engineering
> Based on **Software Requirements (3rd Edition) - Karl Wiegers & Joy Beatty**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Every functional requirement must trace back to exactly one business requirement and have at least one test case (1:N:M traceability).**
2. **Prohibit ambiguous linguistic quantifiers in specifications ('user-friendly', 'fast', 'scalable', 'secure', 'appropriate') without quantifiable metrics.**
3. **Enforce strict RFC 2119 / IEEE 830 modal verbs: SHALL (mandatory), SHOULD (strongly recommended), MAY (optional).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define three-tier requirements (Business, User, Functional). Assign Planguage benchmarks (Scale, Meter, Target) to all non-functional attributes. Enforce full traceability in the RTM.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Specifying implementation details in business requirements.**
- **Orphaned functional requirements with no test verification criteria.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wiegers-requirements-engineering"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
