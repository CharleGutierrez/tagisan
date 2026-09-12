---
name: qual-norman-design-everyday-things
description: "Cognitive ergonomics & human-centered design: Gulf of Execution, Gulf of Evaluation, affordances, visual signifiers, conceptual constraints, and feedback loops."
triggers: ["don-norman", "design-of-everyday-things", "norman-doors", "gulf-of-execution", "gulf-of-evaluation", "affordance", "signifiers", "feedback-loop"]
---

# qual-norman-design-everyday-things
> Based on **The Design of Everyday Things - Don Norman**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Bridge the Gulf of Evaluation in < 100ms through immediate, deterministic visual state changes and audio/haptic cues.**
2. **ALWAYS: Every interactive control must provide unambiguous signifiers (elevation, cursor changes, hover states) matching its underlying affordance.**
3. **NEVER: Create 'Norman doors'—interactive components whose physical appearance contradicts their functional behavior.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Make action possibilities visible and immediate. Provide deterministic feedback for all state transitions so the user never wonders if an action succeeded.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Unclickable-looking buttons or clickable static text.**
- **Asynchronous background mutations with zero loading indicator or feedback.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-norman-design-everyday-things"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
