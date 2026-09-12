---
name: fin-koller-mckinsey-valuation
description: "Enterprise discounted cash flow (DCF) valuation: Return on Invested Capital (ROIC), economic profit, WACC derivation, continuing value, and value creation drivers."
triggers: ["mckinsey-valuation", "tim-koller", "enterprise-dcf", "roic", "economic-profit", "wacc", "continuing-value", "nopat"]
---

# fin-koller-mckinsey-valuation
> Based on **Valuation: Measuring and Managing the Value of Companies - McKinsey & Company (Tim Koller, Marc Goedhart, David Wessels)**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calculate Free Cash Flow to Firm (FCFF) explicitly as: NOPAT + D&A - Delta NWC - CapEx.**
2. **ALWAYS: Economic value is created only when Return on Invested Capital (ROIC) exceeds Weighted Average Cost of Capital (WACC).**
3. **NEVER: Use EBITDA or Net Income as a substitute for free cash flow in corporate valuation models.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model enterprise valuation using discounted FCFF at WACC. Validate that economic profit is positive only when ROIC > WACC.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using EBITDA multiples without accounting for CapEx reinvestment.**
- **Discounting equity cash flows by WACC.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-koller-mckinsey-valuation"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
