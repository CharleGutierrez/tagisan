---
name: ba-volere-requirements-specification
description: "Volere Requirements Process: Volere Snow Card, quantifiable Fit Criteria, customer satisfaction/dissatisfaction gradients, and event-driven elicitation."
triggers: ["volere-requirements-specification", "volere", "fit-criteria", "robertson-requirements", "volere-snow-card", "quantifiable-requirements"]
---

# ba-volere-requirements-specification
> Based on **Mastering the Requirements Process (3rd Edition) - Suzanne Robertson & James Robertson**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **A requirement is undefined until its Fit Criterion is formulated: Fit Criterion = a concrete, unambiguous measurement test that determines whether a solution satisfies the requirement.**
2. **Customer Satisfaction (1 to 5) and Customer Dissatisfaction (1 to 5) gradients must be assigned to distinguish delighters from table-stakes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Every requirement must use the Volere Snow Card: Requirement #, Type, Event, Description, Rationale, Source, Fit Criterion, Customer Satisfaction/Dissatisfaction.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Accepting subjective requirements that cannot be verified by an automated test.**
- **Conflating product desires with statutory constraints.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "volere-requirements-specification"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
