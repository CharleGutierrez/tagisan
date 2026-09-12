---
name: ba-cagan-four-product-risks
description: "The Four Big Product Risks: Value Risk (will they choose it?), Usability Risk (can they use it?), Feasibility Risk (can we build it?), and Business Viability Risk (compliance/finance/legal)."
triggers: ["cagan-four-product-risks", "four-product-risks", "marty-cagan", "inspired", "value-risk", "viability-risk", "feasibility-risk"]
---

# ba-cagan-four-product-risks
> Based on **Inspired: How to Create Tech Products Customers Love - Marty Cagan**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Four Big Risks Assessment: Before writing production code, explicitly validate: Value, Usability, Feasibility, and Business Viability.**
2. **Feasibility vs Viability: AI solves feasibility quickly; the human analyst must rigorously police viability (statutory laws, privacy acts, security).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit every major feature against the 4 Risks: Value, Usability, Feasibility, and Business Viability (legal, compliance, financial, privacy).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Starting coding when only feasibility is understood while ignoring statutory viability risks.**
- **Allowing AI to guess legal compliance rules.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cagan-four-product-risks"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
