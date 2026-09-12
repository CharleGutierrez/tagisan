---
name: fin-hasbrouck-empirical-microstructure
description: "Statistical econometric microstructure: Vector Autoregressions (VAR) of trades and quotes, Kyle's lambda price impact, and Hasbrouck information shares."
triggers: ["joel-hasbrouck", "empirical-microstructure", "kyles-lambda", "price-impact-regression", "var-quotes", "information-share", "tick-econometrics"]
---

# fin-hasbrouck-empirical-microstructure
> Based on **Empirical Market Microstructure: The Institutions, Models, and Econometrics of High-Frequency Trading - Joel Hasbrouck**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Quantify permanent price impact using Kyle's Lambda (Delta P_t = lambda * Order_Flow_t + epsilon_t) to calibrate transaction costs.**
2. **ALWAYS: Estimate cross-venue price discovery using Hasbrouck's information share decomposition.**
3. **NEVER: Assume linear liquidity scaling across varying order trade sizes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Measure empirical price impact using Kyle's Lambda regression: calibrate trade execution simulator slippage against tick-level order book dynamics.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using constant slippage percentages regardless of trade size or volume.**
- **Assuming identical price discovery contribution across fragmented venues.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-hasbrouck-empirical-microstructure"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
