---
name: fin-berk-corporate-finance
description: "Modern corporate finance: The Law of One Price, no-arbitrage condition, capital structure trade-off theory, and dividend policy."
triggers: ["berk-demarzo", "law-of-one-price", "no-arbitrage", "capital-structure-tradeoff", "dividend-irrelevance", "effective-tax-shield"]
---

# fin-berk-corporate-finance
> Based on **Corporate Finance - Jonathan Berk & Peter DeMarzo**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Enforce the Law of One Price across synthetic asset packages; equivalent cash flows must possess identical market valuations.**
2. **ALWAYS: Value debt tax shields by discounting interest tax savings at the cost of debt (r_d) or unlevered cost of capital.**
3. **NEVER: Model leverage increases as costless value creation without factoring in financial distress costs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Verify absence of arbitrage across financial packages: enforce the Law of One Price and calibrate capital structure tradeoffs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Allowing synthetic options or cash flows to price differently than their component parts.**
- **Treating debt financing as having zero distress cost.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-berk-corporate-finance"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
