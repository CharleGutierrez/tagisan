---
name: qual-fogg-tiny-habits
description: "The Fogg Behavior Model: B = M * A * P (Behavior happens when Motivation, Ability, and a Prompt converge). The Ability Curve and tiny action recipes."
triggers: ["bj-fogg", "tiny-habits", "fogg-behavior-model", "behavior-design", "ability-curve", "prompt-timing", "activation-friction"]
---

# qual-fogg-tiny-habits
> Based on **Tiny Habits: The Small Changes That Change Everything - BJ Fogg**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Optimize for user activation by maximizing Ability (reducing friction to < 30 seconds and < 2 clicks) rather than relying on high Motivation.**
2. **ALWAYS: Place Prompts (triggers) only where and when the user has both the Motivation and the immediate Ability to complete the action.**
3. **NEVER: Demand complex user inputs or multi-step tasks at moments of low motivation or high environmental distraction.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maximize user ability by eliminating interaction steps. Ensure prompts coincide exactly with the user's capability and context to act immediately.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Requiring email verification before a user can test a basic search query.**
- **Triggering survey prompts while a user is actively debugging a production error.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-fogg-tiny-habits"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
