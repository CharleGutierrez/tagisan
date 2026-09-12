---
name: fin-joshi-mathematical-finance
description: "Practical financial engineering: Martingale pricing, change of numeraire, Black-Scholes PDE solutions, and Monte Carlo simulation with variance reduction."
triggers: ["mark-joshi", "mathematical-finance", "martingale-pricing", "change-of-numeraire", "monte-carlo-pricing", "antithetic-variates", "variance-reduction"]
---

# fin-joshi-mathematical-finance
> Based on **The Concepts and Practice of Mathematical Finance - Mark S. Joshi**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Drift-adjust underlying processes to the risk-free rate under the risk-neutral measure in Monte Carlo simulations.**
2. **ALWAYS: Implement variance reduction techniques (antithetic variates, control variates) to accelerate Monte Carlo convergence.**
3. **NEVER: Mix real-world drift parameters (mu) into risk-neutral pricing engines.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Monte Carlo pricing engines with risk-neutral drift calibration, antithetic variates for variance reduction, and exact martingale verification.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Running 10 million raw Monte Carlo paths without variance reduction.**
- **Using real-world expected returns to price derivative claims.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-joshi-mathematical-finance"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
