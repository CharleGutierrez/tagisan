---
name: fin-gaughan-mergers-acquisitions
description: "M&A economics: Operational and financial synergies, transaction structures (cash vs stock), accretion/dilution analysis, takeover defenses, and divestitures."
triggers: ["patrick-gaughan", "mergers-and-acquisitions", "m-and-a", "accretion-dilution", "synergy-valuation", "deal-structuring", "takeover-defense"]
---

# fin-gaughan-mergers-acquisitions
> Based on **Mergers, Acquisitions, and Corporate Restructurings - Patrick A. Gaughan**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calculate M&A EPS accretion/dilution accounting for purchase price, financing mix (cash/debt/stock), foregone interest, and goodwill.**
2. **ALWAYS: Discount projected operational synergies with a higher discount rate than base business cash flows due to execution risk.**
3. **NEVER: Justify M&A acquisition premiums with speculative, un-phased synergy projections.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build M&A accretion/dilution models: simulate cash vs stock financing, cost of debt, and tax-effected operational synergy schedules.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming 100% of projected cost synergies materialize on day one.**
- **Ignoring the interest cost of debt used to finance cash acquisitions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-gaughan-mergers-acquisitions"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
