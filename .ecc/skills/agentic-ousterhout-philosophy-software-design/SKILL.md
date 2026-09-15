---
name: agentic-ousterhout-philosophy-software-design
description: "Deep modules vs shallow modules, information hiding, complexity as a symptom of dependency and obscurity, and strategic vs tactical programming."
triggers: ["ousterhout", "philosophy-software-design", "deep-modules", "information-hiding", "tactical-tornado", "strategic-programming"]
---

# agentic-ousterhout-philosophy-software-design
> Based on **A Philosophy of Software Design - John Ousterhout**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Deep Module Invariant: The best modules provide powerful functionality through simple, compact interfaces (deep); avoid shallow modules.**
2. **Strategic vs Tactical Programming: Tactical: quick patches that add technical debt; Strategic: investing 10-20% extra effort in clean design.**
3. **Complexity Definition: Complexity is anything related to the structure of a software system that makes it hard to understand and modify.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Synthesize deep modules with simple public APIs that hide substantial internal complexity. Avoid shallow wrapper classes that increase obscurity.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Becoming a 'tactical tornado' that hacks in quick fixes while degrading overall codebase structure.**
- **Creating shallow interfaces that expose internal implementation details to callers.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ousterhout"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
