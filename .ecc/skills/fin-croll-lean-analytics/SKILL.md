---
name: fin-croll-lean-analytics
description: "Startup unit economics: Customer Acquisition Cost (CAC), Lifetime Value (LTV), CAC payback period, cohort analysis, and virality metrics."
triggers: ["alistair-croll", "lean-analytics", "ltv-cac", "cac-payback", "unit-economics", "cohort-analysis", "customer-acquisition-cost", "churn-decay"]
---

# fin-croll-lean-analytics
> Based on **Lean Analytics: Use Data to Build a Better Startup Faster - Alistair Croll & Benjamin Yoskovitz**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Enforce healthy unit economics thresholds: LTV / CAC >= 3.0 and CAC Payback Period <= 12 months.**
2. **ALWAYS: Calculate LTV using gross profit margin: LTV = (ARPU * Gross Margin %) / Churn Rate.**
3. **NEVER: Calculate LTV using gross top-line revenue without deducting the cost of goods sold (COGS) and cloud hosting fees.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Calculate unit economics using margin-adjusted LTV. Enforce automated budget throttling if CAC payback period exceeds 14 months.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Reporting LTV based on gross revenue instead of gross margin.**
- **Excluding fully loaded sales and marketing payroll from CAC.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-croll-lean-analytics"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
