---
name: fin-fridson-financial-statement-analysis
description: "Credit and cash flow analysis: Flaws of EBITDA, debt service coverage ratio (DSCR), capital structure stress testing, and cash flow waterfall modeling."
triggers: ["martin-fridson", "fridson-alvarez", "ebitda-flaws", "dscr", "debt-service-coverage", "cash-flow-available-debt-service", "cfads"]
---

# fin-fridson-financial-statement-analysis
> Based on **Financial Statement Analysis: A Practitioner's Guide - Martin S. Fridson & Fernando Alvarez**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Model debt service capacity using Cash Flow Available for Debt Service (CFADS) after mandatory maintenance CapEx.**
2. **ALWAYS: Require a minimum Debt Service Coverage Ratio (DSCR) of >= 1.25 for non-speculative debt issuance.**
3. **NEVER: Underwrite debt lines based solely on adjusted EBITDA without modeling working capital drains.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Construct debt service coverage models using cash available for debt service (CFADS). Restrict credit limits if DSCR < 1.25.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying on EBITDA as a cash flow proxy in capital-intensive industries.**
- **Ignoring working capital absorption during revenue expansion.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-fridson-financial-statement-analysis"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
