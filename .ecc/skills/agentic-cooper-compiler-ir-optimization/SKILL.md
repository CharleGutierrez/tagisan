---
name: agentic-cooper-compiler-ir-optimization
description: "Intermediate representations (IR), control flow graphs (CFG), SSA (Static Single Assignment) form, dead code elimination, and register allocation."
triggers: ["cooper", "torczon", "compiler-optimization", "intermediate-representation", "control-flow-graph", "ssa-form", "dead-code-elimination"]
---

# agentic-cooper-compiler-ir-optimization
> Based on **Engineering a Compiler - Keith D. Cooper & Linda Torczon**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Static Single Assignment (SSA): Every variable is assigned exactly once; phi-nodes resolve values at confluence points in the Control Flow Graph.**
2. **Dominator Tree Invariant: Node d dominates node n (d dom n) if every path from entry to n must pass through d.**
3. **Dead Code Elimination: Iteratively removing operations whose definitions have no uses and produce no observable side-effects.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Analyze code refactorings at the Control Flow Graph and SSA level. Verify that transformations preserve dominance invariants and eliminate unreachable dead branches.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Refactorings that leave dangling unused variables, unreferenced imports, or unreachable code blocks.**
- **Accidentally altering phi-node value resolution across branching conditionals.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cooper"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
