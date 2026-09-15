---
name: agentic-dorigo-ant-colony
description: "Pheromone deposition, evaporation dynamics, artificial ant graph routing, combinatorial optimization, and heuristic search across solution spaces."
triggers: ["dorigo", "ant-colony", "aco-optimization", "pheromone-evaporation", "graph-routing", "combinatorial-search"]
---

# agentic-dorigo-ant-colony
> Based on **Ant Colony Optimization - Marco Dorigo & Thomas Stützle**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Pheromone Evaporation: tau_ij(t+1) = (1 - rho) * tau_ij(t) + sum(Delta_tau_ij^k), where rho in (0, 1] is evaporation rate.**
2. **Probabilistic Transition Rule: p_ij^k = [tau_ij]^alpha * [eta_ij]^beta / sum([tau_il]^alpha * [eta_il]^beta), trading off trail history vs local heuristic.**
3. **Pheromone Reinforcement: Successful solution paths deposit reinforcement inversely proportional to path length or compilation cost.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Route agent problem solving through complex call graphs and dependency trees using Ant Colony Optimization. Evaporate failed solution attempts and reinforce verified green test paths.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Zero evaporation rate causing premature lock-in on suboptimal legacy code paths.**
- **Ignoring local heuristic eta_ij (e.g. type checking error count), relying blindly on pheromones.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "dorigo"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
