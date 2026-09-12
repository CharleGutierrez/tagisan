---
name: fin-lyuu-financial-engineering
description: "Computational financial algorithms: Exact decimal arithmetic, banker's rounding, lattice option pricing, yield solvers, and cash flow schedule generation."
triggers: ["yuh-dauh-lyuu", "financial-engineering-computation", "exact-decimal", "bankers-rounding", "yield-solver", "cash-flow-schedule", "fixed-point-arithmetic"]
---

# fin-lyuu-financial-engineering
> Based on **Financial Engineering and Computation: Principles, Mathematics, Algorithms - Yuh-Dauh Lyuu**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Use arbitrary-precision decimal or 64/128-bit integer cents with Banker's Rounding (Half-Even) for all currency calculations.**
2. **ALWAYS: Ensure amortization schedules preserve zero-balance conservation at maturity.**
3. **NEVER: Use floating-point representations (f32/f64, JavaScript number) for currency balances, fees, or interest accruals.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement financial calculations using arbitrary-precision decimal or 128-bit integer cents with banker's rounding (Half-Even), banning IEEE-754 floats.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Storing account balances in IEEE-754 binary floats.**
- **Accumulating round-off errors across millions of transactions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-lyuu-financial-engineering"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
