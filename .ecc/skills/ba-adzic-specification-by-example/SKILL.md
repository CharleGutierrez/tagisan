---
name: ba-adzic-specification-by-example
description: "Executable specifications and living documentation: Deriving scope from goals, illustrating requirements using concrete examples, and single-source-of-truth test suites."
triggers: ["adzic-specification-by-example", "specification-by-example", "living-documentation", "gojko-adzic", "executable-specifications"]
---

# ba-adzic-specification-by-example
> Based on **Specification by Example - Gojko Adzic**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Illustrate rules using concrete data examples rather than abstract formulas.**
2. **Living Documentation Invariant: The specification and the regression test suite must be the exact same artifact.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Before implementing business logic, construct tabular concrete examples showing exact input vectors and expected output values.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing abstract requirements without concrete input/output test vectors.**
- **Letting documentation drift from test suites.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "adzic-specification-by-example"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
