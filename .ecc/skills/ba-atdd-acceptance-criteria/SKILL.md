---
name: ba-atdd-acceptance-criteria
description: "Acceptance Test-Driven Development (ATDD): Test-first specification, boundary value analysis, pass/fail gating, and customer acceptance criteria."
triggers: ["atdd-acceptance-criteria", "atdd", "acceptance-test-driven-development", "ken-pugh", "acceptance-criteria", "pass-fail-gates"]
---

# ba-atdd-acceptance-criteria
> Based on **ATDD by Example - Ken Pugh & Lisa Crispin**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Test-First Verification Gate: Automated acceptance tests must be written, run, and proven failing *before* production code is written.**
2. **Boundary Value Analysis: Every numeric or range constraint must be tested at minimum, maximum, and outside boundaries (e.g., n-1, n, n+1).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Write automated acceptance tests before writing feature code. Verify tests fail for the right reason, then write minimal code to pass.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing tests after the code is finished (confirmation bias).**
- **Testing only happy-path scenarios and ignoring boundary limits.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "atdd-acceptance-criteria"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
