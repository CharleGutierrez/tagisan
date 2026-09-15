---
name: agentic-kahneman-dual-process-thinking
description: "System 1 (heuristic, rapid, intuitive) vs System 2 (deliberative, analytical, slow), cognitive biases, anchoring, loss aversion, and metacognition."
triggers: ["kahneman", "thinking-fast-and-slow", "system-1-system-2", "dual-process-theory", "cognitive-biases", "deliberative-reasoning"]
---

# agentic-kahneman-dual-process-thinking
> Based on **Thinking, Fast and Slow - Daniel Kahneman**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Dual-Process Architecture: System 1 operates automatically and quickly with little or no effort; System 2 allocates attention to effortful mental operations.**
2. **Substitution Heuristic: When faced with a difficult question, System 1 substitutes an easier question without the system noticing.**
3. **Anchoring & Confirmation Bias: The tendency to over-rely on the first piece of information encountered and seek confirming evidence.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use fast System 1 generation for routine boilerplate syntax, but invoke explicit deliberative System 2 verification before committing architecture or security changes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Letting System 1 handle safety-critical concurrency logic, yielding subtle race conditions.**
- **Over-thinking simple formatting tasks with heavyweight System 2 reasoning chains.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kahneman"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
