---
name: agentic-sicp-evaluator-metacircular
description: "Metacircular evaluators, homoiconicity, lexical closures, higher-order functional abstractions, stream processing, and lazy evaluation."
triggers: ["sicp", "abelson-sussman", "metacircular-evaluator", "homoiconicity", "lexical-closures", "higher-order-functions", "lazy-evaluation"]
---

# agentic-sicp-evaluator-metacircular
> Based on **Structure and Interpretation of Computer Programs - Harold Abelson & Gerald Jay Sussman**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **The Eval-Apply Cycle: Eval evaluates expressions relative to an environment; Apply applies procedures to arguments, closing the metacircular loop.**
2. **Lexical Closures: Functions capture their enclosing environment bindings at definition time, maintaining state without global mutations.**
3. **Streams as Infinite Data Structures: Decoupling the simulation of time from the order of events using delayed evaluation (lazy memoization).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Harness higher-order abstractions and closures to build modular agent middleware. Use lazy stream evaluation to process massive code bases incrementally.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying on mutable global variables rather than pure functional closures.**
- **Eagerly loading massive files into memory when streaming generators avoid out-of-memory errors.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sicp"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
