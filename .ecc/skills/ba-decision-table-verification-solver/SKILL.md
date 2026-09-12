---
name: ba-decision-table-verification-solver
description: "Automated Decision Table Verification: SAT/SMT solver principles, detecting rule overlap, finding completeness gaps, and shadowed rule elimination."
triggers: ["decision-table-verification-solver", "decision-table-solver", "rule-overlap-detection", "shadowed-rules", "formal-verification", "sat-solver"]
---

# ba-decision-table-verification-solver
> Based on **Formal Logic & Automated Decision Verification - Silver, Taylor, Ross**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Overlap Detection: Prohibit any two rules in a Unique hit policy table from both evaluating to true for the same input vector.**
2. **Shadowed Rule Elimination: Flag and eliminate any rule whose conditions are a strict subset of an earlier rule that takes precedence.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Run automated SAT/solver checks on decision tables: Verify zero rule overlaps, zero completeness gaps, and zero shadowed rules.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Deploying decision tables that contain dead rules shadowed by earlier catch-all rows.**
- **Manual eyeball review of tables with > 10 input variables.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "decision-table-verification-solver"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
