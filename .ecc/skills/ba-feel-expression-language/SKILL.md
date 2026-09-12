---
name: ba-feel-expression-language
description: "FEEL Expression Language: Strongly-typed expressions, numerical intervals ([a..b], (a..b)), disjunctions, temporal dates/durations, and null-safe navigations."
triggers: ["feel-expression-language", "feel", "dmn-feel", "friendly-enough-expression-language", "dmn-expressions", "range-syntax"]
---

# ba-feel-expression-language
> Based on **Friendly Enough Expression Language (FEEL) - OMG DMN Standard**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **FEEL Type Safety: Strongly typed expressions supporting numbers, strings, booleans, dates, times, durations, and lists.**
2. **Interval Semantics: `[a..b]` (inclusive), `(a..b)` (exclusive), `[a..b)` (inclusive-exclusive), `> x`, `<= y`.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use standard FEEL expressions for all rule conditions: Numerical intervals ([100..500]), list memberships, date comparisons.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using language-specific scripting expressions (e.g. JavaScript eval) inside business rules.**
- **Failing to account for NULL or undefined inputs in FEEL logic.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "feel-expression-language"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
