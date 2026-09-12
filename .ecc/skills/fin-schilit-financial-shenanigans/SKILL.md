---
name: fin-schilit-financial-shenanigans
description: "Forensic accounting: Detecting earnings manipulation, cash flow gimmicks, premature revenue recognition, and hidden liabilities."
triggers: ["howard-schilit", "financial-shenanigans", "accounting-fraud", "earnings-manipulation", "dso-jump", "premature-revenue", "cash-flow-shenanigans"]
---

# fin-schilit-financial-shenanigans
> Based on **Financial Shenanigans: How to Detect Accounting Gimmicks & Fraud in Financial Reports - Howard M. Schilit, Jeremy Perler, Yoni Engelhart**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Trigger forensic audit alerts when Accounts Receivable grows significantly faster than Revenue (Delta AR > Delta Revenue).**
2. **ALWAYS: Reconcile Operating Cash Flow against Net Income; chronic divergence indicates aggressive accruals or artificial boosting.**
3. **NEVER: Recognize revenue before contract obligations are irrevocably delivered and collection is reasonably assured.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build forensic audit rules: flag divergence between operating cash flows and net income, sudden jumps in DSO, and off-balance-sheet liabilities.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Booking revenue upon contract signature before service delivery.**
- **Capitalizing routine operating expenses as intangible software assets.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-schilit-financial-shenanigans"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
