---
name: agentic-brooks-mythical-man-month
description: "Brooks's Law, conceptual integrity, the surgical team, second-system effect, and essential vs accidental complexity in engineering projects."
triggers: ["brooks", "mythical-man-month", "brooks-law", "conceptual-integrity", "surgical-team", "second-system-effect", "essential-complexity"]
---

# agentic-brooks-mythical-man-month
> Based on **The Mythical Man-Month: Essays on Software Engineering - Frederick P. Brooks Jr.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Brooks's Law: Adding manpower to a late software project makes it later (due to combinatorial communication overhead: n*(n-1)/2).**
2. **Conceptual Integrity: The most important attribute of software design; best achieved when a system reflects a single unified architectural vision.**
3. **The Surgical Team: Structuring development teams around a chief architect/developer supported by specialized assistants (toolsmith, tester, editor).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Act as the chief architect's surgical team assistant. Preserve conceptual integrity across all modules and prevent communication bloat.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Spawning dozens of uncoordinated agent workers that create combinatorial git conflicts (Brooks's Law in multi-agent systems).**
- **Falling victim to the second-system effect by packing excessive bells and whistles into a redesign.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "brooks"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
