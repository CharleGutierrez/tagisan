---
name: fin-klarman-margin-of-safety
description: "Risk-averse value investing: Downside protection, complexity arbitrage, forced sellers, illiquidity discounts, and cash as an option."
triggers: ["seth-klarman", "margin-of-safety-klarman", "forced-sellers", "downside-protection", "complexity-arbitrage", "illiquidity-discount", "cash-optionality"]
---

# fin-klarman-margin-of-safety
> Based on **Margin of Safety: Risk-Averse Value Investing Strategies for the Thoughtful Investor - Seth A. Klarman**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Stress-test portfolio valuations against worst-case liquidations with 30-50% illiquidity haircuts and credit freezes.**
2. **ALWAYS: Maintain a dedicated cash reserve as a perpetual call option on emerging distressed market dislocations.**
3. **NEVER: Rely on market liquidity or short-term debt refinancing during periods of systemic market distress.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Execute liquidation scenario stress-tests: model portfolio value under 40% illiquidity haircuts and credit freeze constraints.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming 100% portfolio liquidity in thinly traded secondary markets.**
- **Deploying 100% of capital at market tops with zero liquidity cushion.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-klarman-margin-of-safety"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
