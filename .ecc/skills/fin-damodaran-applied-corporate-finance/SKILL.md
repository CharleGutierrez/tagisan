---
name: fin-damodaran-applied-corporate-finance
description: "Practical corporate finance: Optimal capital structure determination, hurdle rates, dividend vs buyback policy, and returning cash to shareholders."
triggers: ["damodaran-applied", "applied-corporate-finance", "optimal-capital-structure", "wacc-minimization", "dividend-policy", "hurdle-rate-calibration"]
---

# fin-damodaran-applied-corporate-finance
> Based on **Applied Corporate Finance - Aswath Damodaran**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Determine optimal capital structure by minimizing WACC through balancing debt tax shields against expected distress costs.**
2. **ALWAYS: Return excess cash to shareholders (dividends/buybacks) when corporate ROIC falls below the hurdle rate.**
3. **NEVER: Hoard excess cash balances when internal investment opportunities fail to earn the cost of capital.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Calculate optimal capital structure: evaluate debt-to-equity ratios that minimize WACC while keeping credit ratings within investment-grade bounds.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Retaining capital to invest in low-return pet projects.**
- **Taking on debt beyond the WACC-minimizing distress threshold.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-damodaran-applied-corporate-finance"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
