---
name: fin-fabozzi-fixed-income-handbook
description: "Institutional debt securities: Corporate bonds, munis, mortgage-backed securities (MBS prepayment risk), credit default swaps, and structured notes."
triggers: ["fabozzi-handbook", "fixed-income-securities", "mbs", "prepayment-risk", "option-adjusted-spread", "oas", "negative-convexity", "credit-spreads"]
---

# fin-fabozzi-fixed-income-handbook
> Based on **The Handbook of Fixed Income Securities - Frank J. Fabozzi**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Decompose credit spreads using Option-Adjusted Spread (OAS) to isolate embedded option risk from pure credit risk.**
2. **ALWAYS: Account for negative convexity and prepayment contraction/extension risk in mortgage-backed securities.**
3. **NEVER: Price callable or prepayable debt securities without option-adjusted modeling.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model fixed income credit spreads: bootstrap option-adjusted spreads (OAS) and simulate prepayment sensitivity on amortizing collateral.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Valuing callable bonds using standard yield to maturity.**
- **Ignoring prepayment acceleration when interest rates decline.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-fabozzi-fixed-income-handbook"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
