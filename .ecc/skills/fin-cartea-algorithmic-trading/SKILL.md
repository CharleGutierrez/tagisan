---
name: fin-cartea-algorithmic-trading
description: "Optimal trade execution: The Almgren-Chriss framework, balancing market impact risk against inventory timing risk, and optimal liquidation trajectories."
triggers: ["alvaro-cartea", "algorithmic-trading", "almgren-chriss", "optimal-execution", "vwap", "twap", "liquidation-trajectory", "inventory-penalty"]
---

# fin-cartea-algorithmic-trading
> Based on **Algorithmic and High-Frequency Trading - Álvaro Cartea, Sebastian Jaimungal, José Penalva**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Decompose institutional trade execution into TWAP/VWAP schedules using the Almgren-Chriss optimal liquidation framework.**
2. **ALWAYS: Balance temporary market impact costs against the inventory penalty of holding risky positions over time.**
3. **NEVER: Dump large orders into single market orders that cause catastrophic temporary price impact.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement optimal liquidation schedules using Almgren-Chriss equations: calculate execution trajectories balancing temporary market impact against inventory risk.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Executing 10% of daily volume in a single market order.**
- **Ignoring the trade-off between execution speed and market impact.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-cartea-algorithmic-trading"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
