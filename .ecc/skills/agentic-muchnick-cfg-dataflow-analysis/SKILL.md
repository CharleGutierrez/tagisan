---
name: agentic-muchnick-cfg-dataflow-analysis
description: "Dataflow equations (available expressions, reaching definitions, live variables), dominance frontiers, loop transformations, and interprocedural analysis."
triggers: ["muchnick", "dataflow-analysis", "reaching-definitions", "live-variables", "dominance-frontiers", "interprocedural-analysis"]
---

# agentic-muchnick-cfg-dataflow-analysis
> Based on **Advanced Compiler Design and Implementation - Steven S. Muchnick**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Dataflow Equation Framework: Out[B] = Gen[B] union (In[B] \ Kill[B]); In[B] = bigcup_{P in pred(B)} Out[P].**
2. **Liveness Analysis: A variable is live at point p if there exists an execution path from p to a use that does not redefine the variable.**
3. **Monotone Framework Fixed Point: Iterating dataflow equations until convergence is guaranteed by Knaster-Tarski fixed-point theorem on finite lattices.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Perform static dataflow analysis to ensure variables are initialized before use and resources (file handles, network sockets) are safely disposed along all paths.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Introducing uninitialized variable reads along rarely executed error branches.**
- **Resource leaks caused by failing to close handles on abnormal exit paths.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "muchnick"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
