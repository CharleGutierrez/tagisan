---
name: fin-penman-financial-statement-analysis
description: "Accounting-based valuation: Clean surplus accounting, residual earnings model, distinguishing operating activities from financing activities."
triggers: ["stephen-penman", "clean-surplus", "residual-earnings", "operating-vs-financing", "book-value-growth", "accrual-accounting"]
---

# fin-penman-financial-statement-analysis
> Based on **Financial Statement Analysis and Security Valuation - Stephen H. Penman**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Enforce Clean Surplus Accounting: Book Value_t = Book Value_{t-1} + Net Income_t - Dividends_t.**
2. **ALWAYS: Strictly separate Operating Assets/Liabilities from Financing Assets/Liabilities on the balance sheet.**
3. **NEVER: Route transactions through equity adjustments (OCI) without maintaining residual earnings reconciliation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce clean surplus relation across simulated ledger statements: preserve mathematical identity between balance sheet equity and net income.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Burying financing assets inside operating working capital.**
- **Ignoring dirty surplus items that bypass the income statement.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-penman-financial-statement-analysis"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
