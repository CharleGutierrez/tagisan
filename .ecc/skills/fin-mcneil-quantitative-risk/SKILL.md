---
name: fin-mcneil-quantitative-risk
description: "Mathematical risk management: Value at Risk (VaR), Expected Shortfall (CVaR), copulas for tail dependence, and Extreme Value Theory (EVT) for fat tails."
triggers: ["alexander-mcneil", "quantitative-risk-management", "var", "cvar", "expected-shortfall", "extreme-value-theory", "copulas", "tail-dependence"]
---

# fin-mcneil-quantitative-risk
> Based on **Quantitative Risk Management: Concepts, Techniques and Tools - Alexander J. McNeil, Rüdiger Frey, Paul Embrechts**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Complement Value-at-Risk (VaR) with Expected Shortfall (CVaR) at the 99% confidence level to capture tail losses.**
2. **ALWAYS: Model multivariate asset distributions using copulas that accommodate tail dependence; financial returns are fat-tailed and non-Gaussian.**
3. **NEVER: Assume asset return independence or linear correlation structures during systemic market crises.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement multi-asset risk engines using historical simulation and Extreme Value Theory (EVT). Report 99% Expected Shortfall (CVaR) alongside VaR.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying solely on Gaussian VaR models that underestimate 5-sigma crash risk.**
- **Assuming asset correlations remain constant during market sell-offs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-mcneil-quantitative-risk"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
