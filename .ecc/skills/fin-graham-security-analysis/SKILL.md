---
name: fin-graham-security-analysis
description: "The foundational bible of value investing: Intrinsic value, margin of safety, liquidation value (net-nets), bond covenants, and earning power."
triggers: ["graham-dodd", "security-analysis", "margin-of-safety", "intrinsic-value", "net-net", "earning-power", "liquidation-value"]
---

# fin-graham-security-analysis
> Based on **Security Analysis - Benjamin Graham & David L. Dodd**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Enforce a strict Margin of Safety (minimum 30% discount between conservative intrinsic value and entry price).**
2. **ALWAYS: Calculate Net-Net Working Capital value: Cash + 0.75*AR + 0.5*Inventory - Total Liabilities.**
3. **NEVER: Treat price momentum or market enthusiasm as a substitute for verifiable asset backing and earning power.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Incorporate mandatory margin of safety thresholds into asset valuation pipelines: execute buy signals only when price is discounted >= 30% below intrinsic floor.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Buying assets at full intrinsic value with zero margin of safety.**
- **Relying on projected future growth rather than proven historical earning power.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-graham-security-analysis"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
