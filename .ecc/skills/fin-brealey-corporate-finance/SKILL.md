---
name: fin-brealey-corporate-finance
description: "Foundations of corporate financial theory: Net Present Value (NPV) rule, capital budgeting, Modigliani-Miller theorems, and agency costs."
triggers: ["brealey-myers", "corporate-finance", "npv-rule", "capital-budgeting", "modigliani-miller", "internal-rate-of-return", "hurdle-rate"]
---

# fin-brealey-corporate-finance
> Based on **Principles of Corporate Finance - Richard A. Brealey, Stewart C. Myers, Franklin Allen**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Prioritize Net Present Value (NPV) as the primary capital allocation decision rule, rejecting projects with NPV <= 0.**
2. **ALWAYS: Detect and alert on multiple Internal Rates of Return (IRR) when cash flows exhibit non-conventional sign changes.**
3. **NEVER: Rely on payback period or unadjusted accounting rate of return for capital budgeting decisions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement capital budgeting engines using NPV as the primary decision metric. Provide multiple-IRR detection when cash flows flip signs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Accepting projects based on IRR when IRR exceeds hurdle rate but NPV is negative.**
- **Ignoring the time value of money with simple payback periods.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-brealey-corporate-finance"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
