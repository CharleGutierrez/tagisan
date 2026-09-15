---
name: agentic-ghallab-automated-planning
description: "STRIPS and PDDL state representations, preconditions/effects, forward/backward state-space search, and Hierarchical Task Networks (HTN) for multi-step engineering."
triggers: ["ghallab", "automated-planning", "strips", "pddl", "htn-planning", "preconditions-effects", "state-space-search"]
---

# agentic-ghallab-automated-planning
> Based on **Automated Planning: Theory and Practice - Malik Ghallab, Dana Nau & Paolo Traverso**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **STRIPS Action Representation: Action a = (pre(a), add(a), del(a)), updating world state S' = (S \ del(a)) union add(a).**
2. **Hierarchical Task Networks (HTN): Decomposing high-level abstract tasks into partially ordered networks of primitive executable actions.**
3. **Plan Soundness & Completeness: A plan is sound if every action precondition is satisfied and the terminal state satisfies the goal condition.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Formalize complex multi-step coding plans as HTN task trees. Guard every tool invocation with explicit precondition checks (e.g. file exists, git branch clean).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Executing destructive write actions without verifying preconditions, causing irrecoverable workspace corruption.**
- **Flat unorganized task lists that fail to model dependencies between compilation, testing, and deployment.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ghallab"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
