---
name: agentic-newell-unified-cognition
description: "Time scales of human action (biological, cognitive, rational, social bands), problem space hypothesis, and cognitive architecture benchmarks."
triggers: ["newell", "unified-theories-cognition", "problem-space-hypothesis", "cognitive-bands", "cognitive-benchmarks", "action-timescales"]
---

# agentic-newell-unified-cognition
> Based on **Unified Theories of Cognition - Allen Newell**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Bands of Human Action: Biological (~1-10ms), Cognitive (~100ms-10s), Rational (~minutes-hours), and Social (~days-months) time scales.**
2. **Problem Space Hypothesis: All goal-oriented cognitive behavior occurs through search within formulated problem spaces.**
3. **Knowledge Level Principle: An agent's behavior can be predicted solely by knowing its goals and the knowledge it possesses.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure agent tasks across appropriate time scales: sub-second linter fixes at the cognitive band; multi-hour architectural refactors at the rational band.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Applying multi-hour deliberative planning to millisecond-level syntax completions.**
- **Failing to track overarching rational-band goals during low-level cognitive debugging.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "newell"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
