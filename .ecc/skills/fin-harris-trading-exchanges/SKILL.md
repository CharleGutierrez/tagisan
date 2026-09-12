---
name: fin-harris-trading-exchanges
description: "Market microstructure economics: Informed traders, noise traders, arbitrageurs, dealers, adverse selection, and bid-ask spread dynamics."
triggers: ["larry-harris", "trading-and-exchanges", "adverse-selection", "informed-traders", "noise-traders", "dealer-inventory", "effective-spread"]
---

# fin-harris-trading-exchanges
> Based on **Trading and Exchanges: Market Microstructure for Practitioners - Larry Harris**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Measure adverse selection risk (effective spread vs realized spread) in liquidity provisioning algorithms.**
2. **ALWAYS: Widen market-making quotes dynamically when order flow toxicity signals the presence of informed traders.**
3. **NEVER: Quote symmetrical tight bid-ask spreads when inventory imbalances exceed risk limits.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Equip automated market making algorithms with adverse selection protection: widen spreads dynamically when order flow toxicity spikes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Maintaining narrow quotes against toxic flow, accumulating massive adverse inventory.**
- **Treating all order flow as uninformed noise.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-harris-trading-exchanges"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
