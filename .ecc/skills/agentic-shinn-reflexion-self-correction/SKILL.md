---
name: agentic-shinn-reflexion-self-correction
description: "Verbal memory reflection, self-evaluative scalar rewards, error retrospective analysis, trial-and-error retry loops, and episodic memory persistence."
triggers: ["shinn", "reflexion", "verbal-reinforcement", "self-correction", "episodic-reflection", "error-retrospective"]
---

# agentic-shinn-reflexion-self-correction
> Based on **Reflexion: Language Agents with Verbal Reinforcement Learning - Noah Shinn et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Verbal Memory Buffer: Storing structured retrospectives: 'Attempt failed because X. In next attempt, avoid Y and implement Z.'**
2. **Self-Evaluative Heuristic: Evaluating final code artifacts against a scalar rubric (0-100) before presenting them to the user.**
3. **Iterative Correction Loop: Persisting reflection logs across agent iterations to prevent repeating identical failure trajectories.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
When a build or test suite fails, generate an explicit verbal reflection diagnosing the exact failure mechanism before attempting code edits. Store this reflection in episodic memory.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Repeatedly applying the same failing patch across multiple turns without verbal reflection.**
- **Discarding error retrospectives between attempts, losing hard-won debugging context.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "shinn"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
