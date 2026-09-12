---
name: ba-cooper-goal-directed-design
description: "Goal-Directed Interaction Design: User mental models vs implementation models, personas, software posture (sovereign, transient, daemonic), flow, and state preservation."
triggers: ["cooper-goal-directed-design", "about-face", "alan-cooper", "goal-directed-design", "mental-models", "software-posture", "interaction-design"]
---

# ba-cooper-goal-directed-design
> Based on **About Face: The Essentials of Interaction Design (4th Edition) - Alan Cooper et al.**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Mental Model Invariant: The user interface must reflect the user's mental model of their work, NEVER the underlying database implementation model.**
2. **Posture Alignment: Sovereign applications must maximize screen density, keyboard shortcuts, and flow; Transient applications must prioritize instant comprehension.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design UI flows aligned with user mental models. Never expose database IDs or stack traces. Tailor screen layout to application posture.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Exposing internal database schemas directly as form fields.**
- **Interrupting user flow with unnecessary modal dialogs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cooper-goal-directed-design"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
