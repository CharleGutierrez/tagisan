---
name: ba-ross-business-rule-manifesto
description: "Declarative Business Rules & RuleSpeak Grammar: Rules as first-class citizens, separating logic from procedural code, structural vs behavioral rules, and atomic invariants."
triggers: ["ross-business-rule-manifesto", "rulespeak", "business-rule-manifesto", "ronald-ross", "declarative-rules", "policy-rules"]
---

# ba-ross-business-rule-manifesto
> Based on **The Business Rules Manifesto & RuleSpeak - Ronald G. Ross**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Rule Independence: Business rules must be stated declaratively and exist independently of the procedures, screens, or workflows that enforce them.**
2. **RuleSpeak Grammar: 'It is mandatory that [condition]' or 'It is prohibited that [condition]' or 'A [concept] must [constraint]'.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
State all business rules using RuleSpeak declarative syntax. Separate business policy logic from user interface workflows.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Burying business rules in UI click handlers or database triggers.**
- **Writing procedural step-by-step rules instead of declarative invariants.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ross-business-rule-manifesto"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
