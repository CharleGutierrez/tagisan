---
name: fin-damodaran-valuation
description: "Corporate valuation mechanics: Cost of capital, equity risk premiums (ERP), cash flows to firm (FCFF) vs equity (FCFE), and terminal value models."
triggers: ["aswath-damodaran", "damodaran-valuation", "fcff", "fcfe", "cost-of-capital", "cost-of-equity", "beta-unlevering", "equity-risk-premium"]
---

# fin-damodaran-valuation
> Based on **Damodaran on Valuation: Security Analysis for Investment and Corporate Finance - Aswath Damodaran**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Match the discount rate strictly to the cash flow: discount FCFF by WACC, and discount FCFE by the Cost of Equity (r_e).**
2. **ALWAYS: Unlever and relever equity betas using Hamada's equation: Beta_L = Beta_U * (1 + (1 - T) * (D/E)).**
3. **NEVER: Apply Cost of Equity to firm cash flows or WACC to equity cash flows.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Ensure mathematical consistency in DCF calculations: pair firm cash flows with WACC, and equity cash flows with CAPM Cost of Equity.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mixing cash flows to firm with cost of equity.**
- **Using historical betas without debt-unlevering adjustments.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-damodaran-valuation"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
