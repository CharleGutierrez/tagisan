---
name: ba-bpmn-gateway-soundness
description: "Workflow Soundness & Deadlock Freedom: Van der Aalst soundness criteria, liveness, bounded Petri net validation, and gateway matching."
triggers: ["bpmn-gateway-soundness", "workflow-soundness", "marlon-dumas", "deadlock-freedom", "petri-net", "gateway-matching"]
---

# ba-bpmn-gateway-soundness
> Based on **Fundamentals of Business Process Management (2nd Edition) - Dumas, La Rosa, Mendling, Reijers**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Option to Complete: For every reachable state, it is always possible to reach the terminal end state.**
2. **Proper Completion: When the terminal end state is reached, no tokens remain active in any other branch of the process.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit all process flowcharts for Van der Aalst Soundness: (1) Liveness, (2) Proper Completion, (3) Option to complete. Eliminate potential deadlocks or token leaks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Creating cyclic loops without terminating exit guards.**
- **Mismatched gateways that cause token accumulation or thread starvation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bpmn-gateway-soundness"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
