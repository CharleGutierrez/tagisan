---
name: ba-cockburn-use-case-modeling
description: "Goal-oriented use case modeling: Goal levels (Cloud, Sea-Level, Fish), Main Success Scenario, Extension branches, Minimal and Success Guarantees."
triggers: ["cockburn-use-case-modeling", "cockburn", "use-case-modeling", "sea-level-goal", "main-success-scenario", "preconditions-guarantees"]
---

# ba-cockburn-use-case-modeling
> Based on **Writing Effective Use Cases - Alistair Cockburn**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Sea-Level Goal Invariant: User task use cases must represent an atomic business interaction delivering immediate value to the primary actor in a single session.**
2. **Complete Branch Coverage: Every failure or alternate branch must specify an extension step (e.g., 3a, 3b) and define whether it resumes or aborts the Main Success Scenario.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Format use cases with Cockburn standard: Primary Actor, Scope, Level (Sea-level), Preconditions, Minimal Guarantee, Success Guarantee, Main Success Scenario (1-N), Extensions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing use cases at the fish/clam level for single button clicks.**
- **Omitting minimal guarantees for failure scenarios.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cockburn-use-case-modeling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
