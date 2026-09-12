---
name: fin-graham-financial-statements
description: "Classic financial ratio analysis: Balance sheet solvency, working capital adequacy, current/quick ratios, and Net Current Asset Value (NCAV)."
triggers: ["graham-meredith", "interpretation-financial-statements", "ncav", "net-current-asset-value", "working-capital-adequacy", "quick-ratio", "current-ratio"]
---

# fin-graham-financial-statements
> Based on **The Interpretation of Financial Statements - Benjamin Graham & Spencer B. Meredith**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calculate Current Ratio (Current Assets / Current Liabilities >= 2.0) and Quick Ratio (>= 1.0) for solvency verification.**
2. **ALWAYS: Determine Net Current Asset Value (NCAV) by subtracting total liabilities and preferred stock from current assets.**
3. **NEVER: Count goodwill or unamortized intangible assets in tangible book value evaluations.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Compute conservative solvency metrics: isolate Net Current Asset Value (NCAV) and strip all intangible assets in downside credit evaluations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Including goodwill in liquid asset buffers.**
- **Allowing current liabilities to exceed current assets without liquidity warnings.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-graham-financial-statements"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
