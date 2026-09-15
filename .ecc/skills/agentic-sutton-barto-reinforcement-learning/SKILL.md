---
name: agentic-sutton-barto-reinforcement-learning
description: "Bellman equations, Markov Decision Processes (MDP), temporal-difference learning, policy gradients, exploration-exploitation trade-off, and credit assignment."
triggers: ["sutton-barto", "reinforcement-learning", "bellman-equation", "markov-decision-process", "temporal-difference", "policy-gradient", "exploration-exploitation"]
---

# agentic-sutton-barto-reinforcement-learning
> Based on **Reinforcement Learning: An Introduction (2nd ed) - Richard S. Sutton & Andrew G. Barto**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Bellman Optimality Equation: V*(s) = max_a sum_{s', r} p(s', r | s, a) * [r + gamma * V*(s')].**
2. **Temporal-Difference Error: delta_t = R_{t+1} + gamma * V(S_{t+1}) - V(S_t), enabling learning without complete episode rollout.**
3. **Epsilon-Greedy Exploration: Balance exploiting known green test paths with epsilon probability of exploring novel architectural approaches.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Score agent code modifications using explicit reward functions (compiler clean = +10, test pass = +50, regression = -100). Apply credit assignment to isolate failing commits.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Setting discount factor gamma too low, causing myopic fixes that break future extensibility.**
- **Reward hacking where an agent comments out tests to falsely achieve a 100% pass rate.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sutton-barto"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
