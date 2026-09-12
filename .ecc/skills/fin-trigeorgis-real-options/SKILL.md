---
name: fin-trigeorgis-real-options
description: "Strategic real options: Option to expand, abandon, defer, or stage capital investments as Black-Scholes/binomial options."
triggers: ["lenos-trigeorgis", "real-options", "managerial-flexibility", "binomial-lattice", "option-to-expand", "option-to-abandon", "staged-investment"]
---

# fin-trigeorgis-real-options
> Based on **Real Options: Managerial Flexibility and Strategy in Resource Allocation - Lenos Trigeorgis**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Value staged capital investments and technical R&D pipelines as compound real options using binomial decision trees.**
2. **ALWAYS: Recognize that managerial flexibility to expand or abandon adds significant quantifiable economic value over static DCF.**
3. **NEVER: Reject high-uncertainty staged engineering projects with negative static DCF without evaluating embedded real expansion options.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model staged architectural investments using binomial real options lattices, quantifying managerial flexibility to abandon or scale.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using rigid static DCF models for flexible R&D initiatives.**
- **Ignoring the abandonment option value in risky project evaluations.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-trigeorgis-real-options"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
