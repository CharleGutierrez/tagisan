---
name: agentic-silver-mcts-decision-trees
description: "Monte Carlo Tree Search (MCTS), Upper Confidence Bound for Trees (UCT), selection-expansion-simulation-backpropagation loop, and self-play reasoning."
triggers: ["silver", "alphazero", "mcts", "monte-carlo-tree-search", "uct-algorithm", "policy-value-networks", "self-play"]
---

# agentic-silver-mcts-decision-trees
> Based on **Mastering the Game of Go without Human Knowledge / AlphaZero MCTS - David Silver et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **UCT Formula: UCT(v) = Q(v) + c * sqrt(ln(N(parent)) / N(v)), balancing historical win-rate Q with exploration bonus.**
2. **MCTS 4-Phase Loop: 1. Selection (traverse tree via UCT) -> 2. Expansion (add child node) -> 3. Simulation/Evaluation (rollout or value net) -> 4. Backpropagation (update visits and scores).**
3. **Self-Play Alignment: Generating adversarial test cases against synthetic code implementations to discover edge-case regressions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Explore alternative code refactoring branches using MCTS. Rank candidate implementations via automated test-run rollouts and select the branch with the highest cumulative reward.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Greedy depth-first exploration without backtracking, getting stuck in irrecoverable compilation traps.**
- **Zero exploration coefficient (c = 0), preventing the discovery of superior refactoring solutions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "silver"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
