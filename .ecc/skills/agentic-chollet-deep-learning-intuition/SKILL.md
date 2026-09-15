---
name: agentic-chollet-deep-learning-intuition
description: "Representation learning, geometric transformations, generalization vs memorization, gradient descent intuition, and foundational deep learning mechanics."
triggers: ["chollet", "representation-learning", "generalization-memorization", "geometric-transformations", "deep-learning-intuition", "manifold-hypothesis"]
---

# agentic-chollet-deep-learning-intuition
> Based on **Deep Learning with Python - François Chollet**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Manifold Hypothesis: Real-world high-dimensional data concentrates near low-dimensional non-linear manifolds in embedding space.**
2. **Generalization vs Memorization: True intelligence is the ability to adapt to new situations using compact abstraction models rather than memorizing training data.**
3. **Differentiable Programming: Composing differentiable computational graphs optimized via chain-rule backpropagation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design agent reasoning architectures that build generalized mental models of codebases rather than memorizing brittle surface strings. Verify zero-shot edge cases.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Overfitting agent prompts to narrow test inputs, failing on unseen user scenarios.**
- **Treating neural models as infallible databases rather than probabilistic statistical representations.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "chollet"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
