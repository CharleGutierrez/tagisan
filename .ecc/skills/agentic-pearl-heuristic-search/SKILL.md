---
name: agentic-pearl-heuristic-search
description: "A* search admissibility, monotone consistency, branch-and-bound, heuristic pruning, minimax game trees, and computational complexity of heuristics."
triggers: ["pearl", "heuristic-search", "a-star-algorithm", "admissible-heuristic", "monotone-consistency", "branch-and-bound"]
---

# agentic-pearl-heuristic-search
> Based on **Heuristics: Intelligent Search Strategies for Computer Problem Solving - Judea Pearl**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **A* Algorithm Evaluation: f(n) = g(n) + h(n), where g(n) is exact cost from start to n, and h(n) is estimated cost to goal.**
2. **Admissibility & Optimality: If h(n) <= h*(n) (never overestimates true cost), A* is guaranteed to find the optimal path without expanding redundant nodes.**
3. **Consistency (Monotonicity): h(n) <= c(n, a, n') + h(n'), guaranteeing that f-scores along any path never decrease.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement code refactor planners using A* search over AST modification graphs. Ensure the distance heuristic to passing test suites is strictly admissible.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Inadmissible heuristics that prune the true optimal code fix in favor of shallow hacks.**
- **Ignoring duplicate state detection, resulting in infinite loops in cyclic code graphs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "pearl"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
