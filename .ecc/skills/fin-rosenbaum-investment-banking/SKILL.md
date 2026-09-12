---
name: fin-rosenbaum-investment-banking
description: "Wall Street financial modeling: Comparable companies analysis (Comps), Precedent transactions, DCF, and Leveraged Buyout (LBO) debt paydown waterfalls."
triggers: ["rosenbaum-pearl", "investment-banking", "lbo", "lbo-model", "comps", "precedent-transactions", "debt-waterfall", "irr-moic"]
---

# fin-rosenbaum-investment-banking
> Based on **Investment Banking: Valuation, LBOs, M&A, and IPOs - Joshua Rosenbaum & Joshua Pearl**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Build Leveraged Buyout (LBO) models with explicit debt paydown waterfalls, sponsor returns (IRR >= 20%, MoIC >= 2.5x), and covenants.**
2. **ALWAYS: Test LBO returns against exit multiple contraction (exit multiple lower than entry multiple).**
3. **NEVER: Model an LBO without verifying debt covenant compliance (Debt/EBITDA, Interest Coverage) throughout the holding period.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Generate comprehensive LBO model engines: calculate returns (IRR, MoIC) with dynamic debt paydown and revolving credit sweeps.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming exit multiples expand in highly leveraged acquisitions.**
- **Omitting mandatory revolving credit sweeps in LBO cash cascades.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-rosenbaum-investment-banking"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
