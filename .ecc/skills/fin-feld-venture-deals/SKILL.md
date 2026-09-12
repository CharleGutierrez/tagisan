---
name: fin-feld-venture-deals
description: "Venture capital financing: Capitalization tables, pre-money vs post-money, liquidation preferences (participating vs non-participating), option pool shuffle, and anti-dilution."
triggers: ["brad-feld", "jason-mendelson", "venture-deals", "cap-table", "liquidation-preference", "option-pool-shuffle", "anti-dilution", "convertible-notes"]
---

# fin-feld-venture-deals
> Based on **Venture Deals: Be Smarter Than Your Lawyer and Venture Capitalist - Brad Feld & Jason Mendelson**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Model capitalization tables with explicit liquidation preference waterfalls (seniority, participation caps, and common conversion thresholds).**
2. **ALWAYS: Calculate founder ownership dilution accounting for the pre-money unallocated option pool expansion.**
3. **NEVER: Model post-money ownership without applying broad-based weighted-average anti-dilution adjustments in down-rounds.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build capitalization table engines with full waterfall simulation: calculate proceeds across multiple exit valuations, accounting for seniority and anti-dilution.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Promising employee equity percentages without specifying the fully diluted denominator.**
- **Ignoring participating preferred liquidation overhang.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-feld-venture-deals"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
