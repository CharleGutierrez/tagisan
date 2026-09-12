---
name: ba-crud-form-functional-specifications
description: "Functional Form Specifications & State Machines: Form FSM (Pristine, Dirty, Validating, Submitting, Submitted, Error), optimistic UI, inline validation, dirty tracking."
triggers: ["crud-form-functional-specifications", "form-design", "luke-wroblewski", "form-state-machine", "dirty-tracking", "double-submit-protection"]
---

# ba-crud-form-functional-specifications
> Based on **Web Form Design: Filling in the Blanks - Luke Wroblewski**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Double-Submit Protection: While form state is `Submitting`, disable submit buttons and reject duplicate requests.**
2. **Dirty Navigation Guard: If form state is `Dirty` and user navigates away, prompt confirmation to prevent data loss.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement complete Form FSM: Pristine -> Dirty -> Validating -> Submitting -> Submitted / Error. Provide inline validation and dirty navigation guards.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Allowing double-clicks on submit buttons that fire duplicate API requests.**
- **Silently discarding user form inputs when navigating away.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "crud-form-functional-specifications"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
