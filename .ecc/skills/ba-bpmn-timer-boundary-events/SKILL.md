---
name: ba-bpmn-timer-boundary-events
description: "Boundary Events & Exception Flow: Interrupting vs non-interrupting boundary events, timer escalations, error boundaries, compensation, and cancel events."
triggers: ["bpmn-timer-boundary-events", "boundary-events", "timer-event", "error-event", "compensation-event", "bpmn-exceptions"]
---

# ba-bpmn-timer-boundary-events
> Based on **OMG BPMN 2.0 Executable Specification - Object Management Group**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Boundary Event Semantics: An Interrupting Boundary Event cancels the attached activity and diverts token flow; a Non-Interrupting Event spawns a parallel execution branch.**
2. **Compensation Invariant: Compensation events can only be triggered after the associated activity has completed successfully.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Specify SLA timeouts using boundary timer events. Route technical errors and business exceptions via explicit boundary error events.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Letting tasks hang indefinitely without a timer boundary event.**
- **Using interrupting events when a background notification was intended.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bpmn-timer-boundary-events"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
