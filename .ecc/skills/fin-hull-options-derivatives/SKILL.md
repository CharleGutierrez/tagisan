---
name: fin-hull-options-derivatives
description: "The derivatives pricing benchmark: Black-Scholes-Merton model, Greeks (Delta, Gamma, Theta, Vega, Rho), put-call parity, binomial trees, and risk-neutral valuation."
triggers: ["john-hull", "black-scholes", "put-call-parity", "options-greeks", "risk-neutral-valuation", "implied-volatility", "binomial-option-pricing"]
---

# fin-hull-options-derivatives
> Based on **Options, Futures, and Other Derivatives - John C. Hull**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Enforce Put-Call Parity as a strict invariant across option models: Call + K * exp(-rT) = Put + Spot_0.**
2. **ALWAYS: Calculate continuous analytical partial derivatives for option Greeks (Delta, Gamma, Theta, Vega, Rho) rather than noisy finite differences.**
3. **NEVER: Price options using real-world asset drift rates (mu); pricing must occur under the risk-neutral measure (r).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Price financial derivatives using analytical Black-Scholes formulas, verifying put-call parity bounds and computing continuous Greek sensitivities.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using finite difference approximations for simple vanilla Greeks.**
- **Violating put-call parity bounds in pricing or market-making models.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-hull-options-derivatives"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
