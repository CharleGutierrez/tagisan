---
name: fin-grinold-active-portfolio
description: "Quantitative portfolio management: The Fundamental Law of Active Management (IR = IC * sqrt(Breadth)), information ratio, benchmark tracking error, and factor models."
triggers: ["grinold-kahn", "active-portfolio-management", "fundamental-law", "information-ratio", "information-coefficient", "tracking-error", "factor-investing"]
---

# fin-grinold-active-portfolio
> Based on **Active Portfolio Management: A Quantitative Approach for Providing Superior Returns and Controlling Risk - Richard C. Grinold & Ronald N. Kahn**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Calibrate quantitative trading strategies to maximize the Information Ratio (IR = Information Coefficient * sqrt(Breadth)).**
2. **ALWAYS: Measure active portfolio risk strictly against the benchmark using Ex-Ante Tracking Error.**
3. **NEVER: Concentrate portfolio risk in correlated bets while assuming high statistical breadth.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit trading algorithms against the Fundamental Law of Active Management: measure realized Information Coefficient (IC) and calculate portfolio Information Ratio.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Claiming 10,000 independent bets per day when all bets are exposed to a single common risk factor.**
- **Neglecting benchmark tracking error.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-grinold-active-portfolio"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
