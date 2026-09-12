---
name: fin-tzuo-subscribed
description: "The subscription economy: ARR, MRR, churn dynamics, Net Revenue Retention (NRR), expansion revenue, and customer lifetime relationship economics."
triggers: ["tien-tzuo", "subscribed", "subscription-economy", "arr", "mrr", "nrr", "net-revenue-retention", "expansion-revenue", "churn-rate"]
---

# fin-tzuo-subscribed
> Based on **Subscribed: Why the Subscription Model Will Be Your Company's Future - Tien Tzuo & Gabe Weisert**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Isolate Monthly/Annual Recurring Revenue (MRR/ARR) strictly from one-time professional services or non-recurring setup fees.**
2. **ALWAYS: Calculate Net Revenue Retention (NRR): (Beginning ARR + Expansion - Contraction - Churn) / Beginning ARR.**
3. **NEVER: Bundle one-time consulting or custom engineering payments into ARR run-rate multipliers.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Separate recurring subscription billing streams from one-off charges. Calculate Net Revenue Retention (NRR) and cohort expansion dynamics.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Multiplying one-time contract setup fees by 12 to inflate reported ARR.**
- **Ignoring logo churn when net dollar retention is temporarily high.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-tzuo-subscribed"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
