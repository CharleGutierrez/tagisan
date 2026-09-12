---
name: fin-fabozzi-fixed-income-math
description: "Bond pricing mathematics: Yield to maturity (YTM), spot rate bootstrapping, Macaulay duration, modified duration, and convexity."
triggers: ["frank-fabozzi", "fixed-income-math", "yield-to-maturity", "macaulay-duration", "modified-duration", "convexity", "bond-pricing", "spot-curve"]
---

# fin-fabozzi-fixed-income-math
> Based on **Fixed Income Mathematics: Analytical and Statistical Techniques - Frank J. Fabozzi**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calculate both Modified Duration and Convexity for interest rate sensitivity: Delta P / P = -D_mod * Delta y + 0.5 * C * (Delta y)^2.**
2. **ALWAYS: Bootstrap zero-coupon spot rate yield curves from par coupon bond yields.**
3. **NEVER: Rely on duration alone for yield shifts exceeding 100 basis points.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Price fixed income instruments using zero-coupon spot rate curve bootstrapping. Compute modified duration and second-order convexity adjustments.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using linear duration approximations for large interest rate shocks.**
- **Pricing cash flows with a single flat YTM instead of the spot curve.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-fabozzi-fixed-income-math"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
