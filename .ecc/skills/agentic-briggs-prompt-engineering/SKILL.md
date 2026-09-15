---
name: agentic-briggs-prompt-engineering
description: "Chain-of-Thought (CoT), Tree-of-Thoughts (ToT), directional stimulus prompting, few-shot exemplars, prompt injection defenses, and system prompt hardening."
triggers: ["briggs", "ingham", "prompt-engineering", "chain-of-thought", "tree-of-thoughts", "prompt-injection-defense", "system-prompt-hardening"]
---

# agentic-briggs-prompt-engineering
> Based on **Prompt Engineering for Generative AI - James Briggs & Francisco Ingham**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Chain-of-Thought Reasoning: Eliciting intermediate reasoning steps significantly boosts performance on multi-step algorithmic deduction.**
2. **Tree-of-Thoughts (ToT): Exploration of branching thought trajectories evaluated by self-assessment scoring and backtracking.**
3. **Prompt Injection Boundary Delimiters: Enclosing user inputs in strict XML/Markdown fences (<user_input>...</user_input>) to prevent instruction hijacking.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce explicit 'Thinking' blocks before code synthesis. Sanitize and isolate all untrusted inputs with boundary tags and anti-injection instructions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Permitting raw user text to concatenate directly with system instructions without sanitization delimiters.**
- **Skipping scratchpad reasoning on non-trivial algorithmic tasks, leading to logic flaws.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "briggs"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
