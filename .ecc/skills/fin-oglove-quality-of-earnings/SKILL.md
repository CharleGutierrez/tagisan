---
name: fin-oglove-quality-of-earnings
description: "Earnings quality analysis: Operating cash flow vs reported net income, inventory buildup red flags, tax rate distortions, and non-recurring windfalls."
triggers: ["thornton-oglove", "quality-of-earnings", "cash-conversion-ratio", "inventory-buildup", "accrual-quality", "non-recurring-gains"]
---

# fin-oglove-quality-of-earnings
> Based on **Quality of Earnings - Thornton L. O'Glove**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calculate the Cash Conversion Quality as CFO / Net Income; a ratio < 1.0 indicates low earnings quality.**
2. **ALWAYS: Strip non-operating, non-recurring gains (asset sales, legal settlements) from normalized operating earnings.**
3. **NEVER: Treat deferred tax reversals or one-time tax credits as permanent reductions in the effective tax rate.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Compute earnings quality index: continuously compare operating cash flow to net income and strip non-recurring windfalls from normalized run-rates.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Celebrating record net income when operating cash flow is deeply negative.**
- **Extrapolating one-time asset sales into future earnings projections.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-oglove-quality-of-earnings"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
