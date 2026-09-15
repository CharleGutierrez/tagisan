---
name: agentic-nystrom-crafting-interpreters
description: "Tree-walk interpreters, bytecode virtual machines, Pratt parsing, garbage collection, and stack-based execution architectures."
triggers: ["nystrom", "crafting-interpreters", "bytecode-vm", "pratt-parsing", "tree-walk-interpreter", "garbage-collection"]
---

# agentic-nystrom-crafting-interpreters
> Based on **Crafting Interpreters - Robert Nystrom**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Pratt Parsing (Top-Down Operator Precedence): Associating parse functions with token types and binding powers to parse expressions cleanly in O(N).**
2. **Stack-Based VM Dispatch: Executing instructions via a central bytecode evaluation loop manipulating an explicit operand value stack.**
3. **Mark-and-Sweep Garbage Collection: Tracing reachable objects from root references (stack, globals) and reclaiming unreachable memory.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Scaffold internal domain-specific scripting interpreters using Pratt parsing for ergonomic expressions and stack-based bytecode evaluation for execution speed.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing messy recursive-descent parsers for mathematical expressions when Pratt parsing handles precedence cleanly.**
- **Creating circular object references in custom interpreters without cycle-collection support.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "nystrom"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
