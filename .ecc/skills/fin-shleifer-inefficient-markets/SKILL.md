---
name: fin-shleifer-inefficient-markets
description: "The limits of arbitrage: Fundamental risk, noise trader risk, implementation costs, and why prices deviate persistently from fundamental value."
triggers: ["andrei-shleifer", "inefficient-markets", "limits-of-arbitrage", "noise-trader-risk", "behavioral-finance", "arbitrage-limits", "sentiment-risk"]
---

# fin-shleifer-inefficient-markets
> Based on **Inefficient Markets: An Introduction to Behavioral Finance - Andrei Shleifer**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Model funding liquidity constraints and margin call risks in statistical arbitrage algorithms; prices can stay irrational longer than you can stay solvent.**
2. **ALWAYS: Account for noise trader risk in market-making and arbitrage strategy risk models.**
3. **NEVER: Assume infinite balance sheet capacity when arbitraging apparent market mispricings.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Incorporate limits-of-arbitrage constraints in algorithmic trading engines: simulate margin requirements, borrow fees, and noise trader divergence risk.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming arbitrage is risk-free and instantaneous.**
- **Over-leveraging statistical arbitrage positions without margin buffers.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-shleifer-inefficient-markets"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
