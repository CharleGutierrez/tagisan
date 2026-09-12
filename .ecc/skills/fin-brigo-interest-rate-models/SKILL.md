---
name: fin-brigo-interest-rate-models
description: "Advanced interest rate mathematics: Short-rate models (Hull-White, Vasicek), Libor Market Model (LMM), SOFR transition, and swaption volatility smiles."
triggers: ["damiano-brigo", "fabio-mercurio", "interest-rate-models", "hull-white", "short-rate-model", "swaption-smile", "sofr", "term-structure"]
---

# fin-brigo-interest-rate-models
> Based on **Interest Rate Models - Theory and Practice: With Smile, Inflation and Credit - Damiano Brigo & Fabio Mercurio**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calibrate short-rate interest rate models (e.g. 1-factor Hull-White) to exactly fit the current zero-coupon market yield curve.**
2. **ALWAYS: Model interest rate volatility smiles and skews when pricing complex swaptions and caps/floors.**
3. **NEVER: Use uncalibrated random walks for interest rates that permit negative rates without explicit floor mechanics.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Simulate interest rate paths using calibrated 1-factor Hull-White models, ensuring exact fit to current market zero-coupon curves.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using flat interest rate volatility assumptions across tenors.**
- **Failing to calibrate short-rate models to current market discount factors.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-brigo-interest-rate-models"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
