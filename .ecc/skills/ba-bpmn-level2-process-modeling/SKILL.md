---
name: ba-bpmn-level2-process-modeling
description: "Descriptive & Analytic BPMN 2.0: Pools, lanes, event markers, exclusive (XOR), parallel (AND), inclusive (OR) gateways, and token flow semantics."
triggers: ["bpmn-level2-process-modeling", "bpmn", "bruce-silver", "bpmn-method-and-style", "workflow-modeling", "token-flow", "gateways"]
---

# ba-bpmn-level2-process-modeling
> Based on **BPMN Method and Style (2nd Edition) - Bruce Silver**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Gateway Soundness: An Exclusive Gateway (XOR) must split flow into exactly one branch; a Parallel Gateway (AND) split must be paired with an AND join to avoid deadlocks.**
2. **Token Conservation: Every token generated at a Start Event must eventually be consumed at an End Event without leaking or being indefinitely trapped.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model business workflows using BPMN 2.0 rules: Clear Pools/Lanes, Start/End events, matched split/join gateways, and message flows across pool boundaries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using an AND join for paths originating from an XOR split, causing deadlocks.**
- **Drawing sequence flows across pool boundaries (violating BPMN standard).**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bpmn-level2-process-modeling"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
