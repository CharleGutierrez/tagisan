---
name: agentic-anthropic-constitutional-ai
description: "Principle-based self-critique, Reinforcement Learning from AI Feedback (RLAIF), constitutional rulesets, and automated chain-of-thought moderation."
triggers: ["anthropic", "constitutional-ai", "rlaif", "self-critique", "constitutional-rules", "chain-of-thought-moderation"]
---

# agentic-anthropic-constitutional-ai
> Based on **Constitutional AI: Harmlessness from AI Feedback - Yuntao Bai et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Critique and Revision Loop: Generate response -> Critique response against constitution principles -> Revise response to satisfy principles.**
2. **Constitutional Principles: Unambiguous axioms governing safety, copyright, ethical behavior, and software correctness.**
3. **RLAIF Alignment: Training preference models using automated AI critiques based on constitutional criteria rather than manual human labeling.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Equip coding agents with an explicit architectural constitution. Before committing code, have the agent execute a self-critique step against the constitution.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Committing raw first-draft code without a critique and revision pass.**
- **Vague constitutional rules that cannot be objectively verified by automated checks.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "anthropic"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
