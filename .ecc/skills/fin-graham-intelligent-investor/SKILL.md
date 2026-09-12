---
name: fin-graham-intelligent-investor
description: "Defensive vs enterprising investing: Mr. Market allegory, formula investing, mechanical dollar-cost averaging, and price vs value dichotomy."
triggers: ["benjamin-graham", "intelligent-investor", "mr-market", "defensive-investor", "enterprising-investor", "dollar-cost-averaging", "portfolio-rebalancing"]
---

# fin-graham-intelligent-investor
> Based on **The Intelligent Investor - Benjamin Graham**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Implement algorithmic portfolio rebalancing driven by fixed allocation bands (e.g. 50/50 stock/bond with rebalance triggers at +/- 5%).**
2. **ALWAYS: Treat market quotations as offers from 'Mr. Market' to be exploited, never as statements of intrinsic truth.**
3. **NEVER: Allow automated investment workflows to chase market prices during speculative run-ups.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design disciplined programmatic rebalancing engines: rebalance asset weights based on predetermined mechanical thresholds, neutralizing market sentiment.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Abandoning rebalancing rules during market euphoria.**
- **Panic selling automated portfolios at the bottom of market drawdowns.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-graham-intelligent-investor"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
