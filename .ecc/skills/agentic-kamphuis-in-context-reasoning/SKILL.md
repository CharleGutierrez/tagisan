---
name: agentic-kamphuis-in-context-reasoning
description: "In-context scaffolding, role and persona definition, iterative conversational steering, constraint anchoring, and cognitive scaffolding for coding agents."
triggers: ["kamphuis", "in-context-reasoning", "cognitive-scaffolding", "persona-definition", "conversational-steering", "constraint-anchoring"]
---

# agentic-kamphuis-in-context-reasoning
> Based on **The Art of Asking AI - Nathan Kamphuis**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Persona Anchoring: Priming the agent as an elite principal systems engineer focuses the conditional probability distribution toward robust code patterns.**
2. **Negative Constraint Priming: Explicitly enumerating forbidden patterns ('DO NOT use deprecated API X') reduces error rates.**
3. **Iterative Narrowing: Guiding the agent from high-level architectural specification down to function-level implementation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Anchor the agent's persona with precise domain expertise. Clearly state architectural invariants, performance targets, and forbidden dependencies upfront.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Vague, generic prompts ('write code for X') that yield superficial or incomplete implementations.**
- **Stating what to do without stating what NOT to do, allowing anti-patterns to seep in.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kamphuis"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
