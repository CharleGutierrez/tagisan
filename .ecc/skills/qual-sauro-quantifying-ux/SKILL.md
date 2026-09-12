---
name: qual-sauro-quantifying-ux
description: "Practical statistics for user research: System Usability Scale (SUS), Single Ease Question (SEQ), Net Promoter, task completion confidence intervals, and small sample stats."
triggers: ["jeff-sauro", "james-lewis", "quantifying-ux", "system-usability-scale", "sus-score", "single-ease-question", "seq", "ux-statistics"]
---

# qual-sauro-quantifying-ux
> Based on **Quantifying the User Experience - Jeff Sauro & James R. Lewis**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Target an empirical System Usability Scale (SUS) equivalent score >= 80 for production interfaces.**
2. **ALWAYS: Embed single-question micro-telemetry (SEQ: 1 to 7 ease rating) immediately following complex task completion.**
3. **NEVER: Rely on subjective personal opinions to evaluate interaction difficulty when standardized psychometric UX metrics exist.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Incorporate standardized psychometric UX measurement (SUS targets, post-task SEQ capture) directly into application telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Treating UX quality as a purely subjective aesthetic debate.**
- **Failing to measure user perceived effort after complex configuration flows.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-sauro-quantifying-ux"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
