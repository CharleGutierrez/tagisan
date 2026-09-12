---
name: fin-rappaport-creating-shareholder-value
description: "Shareholder Value Added (SVA): Evaluating corporate strategies by cash flows generated rather than accounting earnings (EPS) growth."
triggers: ["alfred-rappaport", "creating-shareholder-value", "sva", "shareholder-value-added", "cash-flow-return-on-investment", "value-growth-duration"]
---

# fin-rappaport-creating-shareholder-value
> Based on **Creating Shareholder Value: A Guide for Managers and Investors - Alfred Rappaport**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Measure strategic value creation using Cumulative Operating Cash Flow minus Capital Reinvestment (Cash Flow Return on Investment).**
2. **ALWAYS: Evaluate business unit strategies by their Value Growth Duration (VGD)—the period over which investments earn returns above cost of capital.**
3. **NEVER: Align executive or algorithmic incentives with short-term accounting earnings per share.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model Shareholder Value Added (SVA): decouple executive performance metrics from accounting EPS, linking incentives directly to discounted cash value created.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Managing business units to hit quarterly EPS targets at the expense of long-term cash flow.**
- **Ignoring capital reinvestment costs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-rappaport-creating-shareholder-value"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
