---
name: fin-damodaran-dark-side-valuation
description: "Valuing high-uncertainty assets: Pre-revenue tech startups, distressed debt, cyclical firms, and failure probability weighting."
triggers: ["dark-side-valuation", "startup-valuation", "distressed-valuation", "failure-probability", "reinvestment-rate", "survival-model"]
---

# fin-damodaran-dark-side-valuation
> Based on **The Dark Side of Valuation: Valuing Young, Distressed, and Complex Businesses - Aswath Damodaran**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Discount startup valuations by explicit survival probabilities: Value = V_going_concern * p + V_liquidation * (1 - p).**
2. **ALWAYS: Bound terminal growth rates by the risk-free rate or the long-term risk-free GDP growth rate.**
3. **NEVER: Extrapolate exponential revenue growth into terminal periods without margin mean-reversion and reinvestment drag.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model startup valuations with dynamic failure probability weighting and realistic reinvestment rates (g = Reinvestment Rate * ROIC).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming 100% startup survival in 10-year forecast models.**
- **Projecting terminal growth rates higher than GDP growth.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-damodaran-dark-side-valuation"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
