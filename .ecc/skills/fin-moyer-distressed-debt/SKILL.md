---
name: fin-moyer-distressed-debt
description: "Bankruptcy and restructuring finance: Priority of claims, Chapter 11 reorganization, the Absolute Priority Rule, fulcrum securities, and structural subordination."
triggers: ["stephen-moyer", "distressed-debt", "absolute-priority-rule", "fulcrum-security", "structural-subordination", "chapter-11", "reorganization-plan"]
---

# fin-moyer-distressed-debt
> Based on **Distressed Debt Analysis: Strategies for Speculative Investors - Stephen G. Moyer**

## 1. Core Financial Foundations & Formal Invariants

1. **ALWAYS: Model debt liquidation and restructuring waterfalls adhering strictly to the Absolute Priority Rule.**
2. **ALWAYS: Identify the 'fulcrum security'—the debt tranche that is partially covered by enterprise value and will convert to control equity.**
3. **NEVER: Assign recovery value to junior equity when senior debt tranches remain impaired and un-reorganized.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Construct distressed debt waterfall models: identify the fulcrum security by simulating asset recovery values across seniority tranches.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assigning positive value to common stock in an insolvent liquidation.**
- **Overlooking structural subordination between parent and operating company debt.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "fin-moyer-distressed-debt"

# Execute automated financial & valuation audit
cargo test --test fin_skills_brutal_tests
```
