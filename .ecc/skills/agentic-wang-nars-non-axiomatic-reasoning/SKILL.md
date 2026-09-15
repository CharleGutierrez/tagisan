---
name: agentic-wang-nars-non-axiomatic-reasoning
description: "Assumption of Insufficient Knowledge and Resources (AIKR), truth values as frequency and confidence, syllogistic inference, and open-world reasoning."
triggers: ["wang", "nars", "non-axiomatic-logic", "aikr", "truth-value-confidence", "syllogistic-reasoning", "open-world-reasoning"]
---

# agentic-wang-nars-non-axiomatic-reasoning
> Based on **Non-Axiomatic Logic: A Model of Intelligent Reasoning (NARS) - Pei Wang**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **AIKR Axiom: An intelligent agent must adapt under the constraint of Insufficient Knowledge and Resources (finite memory, finite compute, real-time deadlines).**
2. **NARS Truth Value Pair: <f, c>, where f in [0, 1] is the frequency of positive evidence, and c in [0, 1) is confidence based on total amount of evidence.**
3. **Non-Axiomatic Syllogism: Deducing, inducting, and abducing relationships between concepts while tracking evidence confidence intervals.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Operate explicitly under AIKR constraints: prioritize high-confidence code suggestions when time is scarce; fall back to conservative approximations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming infinite time and compute to solve coding tasks under strict production deadlines.**
- **Treating uncertain empirical observations as binary (1 or 0) mathematical truths.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "wang"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
