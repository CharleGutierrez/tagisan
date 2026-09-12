---
name: fin-shiller-irrational-exuberance
description: "Speculative price feedback loops: The Cyclically Adjusted Price-to-Earnings (CAPE / Shiller P/E) ratio, cultural drivers, and asset bubble contagion."
triggers: ["robert-shiller", "irrational-exuberance", "cape-ratio", "shiller-pe", "cyclically-adjusted-pe", "speculative-feedback", "mean-reverting-valuation"]
---

# fin-shiller-irrational-exuberance
> Based on **Irrational Exuberance - Robert J. Shiller**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Evaluate long-term equity valuation multiples using the 10-year inflation-adjusted Cyclically Adjusted P/E (CAPE) ratio.**
2. **ALWAYS: Incorporate mean-reverting valuation multiples when forecasting long-term 10-year asset returns.**
3. **NEVER: Project historical trailing equity returns forward when valuation multiples trade at extreme historical percentiles.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Calculate Shiller CAPE ratios across equity indices: calibrate long-term asset return projections to mean-reverting valuation multiples.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Projecting 15% annual equity returns when CAPE is above 35.**
- **Ignoring historical valuation mean-reversion.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-shiller-irrational-exuberance"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
