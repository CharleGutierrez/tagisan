---
name: fin-thorndike-the-outsiders
description: "Capital allocation mastery: Per-share cash flow focus, opportunistic share repurchases, debt paydown, decentralized operations, and tax optimization."
triggers: ["william-thorndike", "the-outsiders", "capital-allocation", "per-share-cash-flow", "share-buybacks", "decentralized-capital", "fcf-per-share"]
---

# fin-thorndike-the-outsiders
> Based on **The Outsiders: Eight Unconventional CEOs and Their Radically Rational Blueprint for Success - William N. Thorndike Jr.**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Measure corporate performance on a per-share basis: Free Cash Flow Per Share (FCF / Fully Diluted Shares).**
2. **ALWAYS: Simulate share repurchases only when market stock price is demonstrably discounted below intrinsic business value.**
3. **NEVER: Reward gross revenue growth that dilutes per-share cash flow.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Track corporate capital allocation KPIs: compute Free Cash Flow Per Share and trigger buyback simulations when market price falls below intrinsic value.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Executing share buybacks at market peak valuations to offset option dilution.**
- **Prioritizing top-line revenue while per-share cash flow shrinks.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-thorndike-the-outsiders"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
