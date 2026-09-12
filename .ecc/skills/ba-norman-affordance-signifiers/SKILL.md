---
name: ba-norman-affordance-signifiers
description: "Design Psychology & Norman Principles: Affordances, signifiers, constraints (physical, cultural, semantic, logical), mappings, feedback, and bridging Gulf of Execution/Evaluation."
triggers: ["norman-affordance-signifiers", "don-norman", "design-of-everyday-things", "affordances", "signifiers", "gulf-of-execution", "immediate-feedback"]
---

# ba-norman-affordance-signifiers
> Based on **The Design of Everyday Things - Don Norman**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Signifier Invariant: Interactive elements must have clear visual signifiers indicating where and how to interact.**
2. **100ms Feedback Rule: The system must provide perceptible feedback for every user action within 100 milliseconds.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Provide visual signifiers for all clickable elements. Deliver state feedback within 100ms. Enforce natural mappings between controls and effects.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Designing clickable buttons that look like static text.**
- **Performing async operations without giving the user loading indicators.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "norman-affordance-signifiers"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
