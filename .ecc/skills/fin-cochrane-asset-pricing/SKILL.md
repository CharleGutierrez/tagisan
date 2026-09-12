---
name: fin-cochrane-asset-pricing
description: "Unified asset pricing theory: Stochastic Discount Factor (SDF: p = E[m * x]), consumption CAPM, factor pricing models, and Generalized Method of Moments (GMM)."
triggers: ["john-cochrane", "asset-pricing", "stochastic-discount-factor", "sdf", "consumption-capm", "gmm", "pricing-kernel"]
---

# fin-cochrane-asset-pricing
> Based on **Asset Pricing - John H. Cochrane**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Frame multi-asset valuation within a coherent stochastic discount factor framework: Price = E[m * Payoff].**
2. **ALWAYS: Ensure that the pricing kernel (m) is strictly non-negative across all states of the world to eliminate arbitrage.**
3. **NEVER: Price assets using ad-hoc multiple heuristics that violate the fundamental theorem of asset pricing.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Validate multi-asset pricing models against the Stochastic Discount Factor (SDF) condition: ensure pricing kernels remain non-negative to guarantee absence of arbitrage.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assigning negative state prices in lattice models.**
- **Using inconsistent discount factors across assets with identical risk exposures.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-cochrane-asset-pricing"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
