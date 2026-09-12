---
name: fin-parker-platform-revolution
description: "Platform economics: Two-sided network effects, platform take rates, Gross Merchandise Value (GMV), transaction velocity, and cross-side subsidies."
triggers: ["geoffrey-parker", "platform-revolution", "two-sided-markets", "network-effects", "gmv", "take-rate", "cross-side-subsidies", "marketplace-economics"]
---

# fin-parker-platform-revolution
> Based on **Platform Revolution: How Networked Markets Are Transforming the Economy - Geoffrey G. Parker, Marshall W. Van Alstyne, Sangeet Paul Choudary**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Distinguish Gross Merchandise Value (GMV) from Net Platform Revenue: Net Revenue = GMV * Take Rate.**
2. **ALWAYS: Verify that the marginal lifetime value of subsidized platform participants exceeds the cost of cross-side subsidies.**
3. **NEVER: Book GMV as corporate revenue; book only the earned platform take-rate commission.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model platform marketplace economics: separate GMV volume from net revenue take-rates, tracking cohort contribution margins per side of the market.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Reporting gross marketplace volume as GAAP revenue.**
- **Providing unsustainable customer acquisition subsidies with zero long-term retention.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-parker-platform-revolution"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
