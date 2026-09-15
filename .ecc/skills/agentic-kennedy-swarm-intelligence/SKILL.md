---
name: agentic-kennedy-swarm-intelligence
description: "Particle Swarm Optimization (PSO), socio-cognitive velocity updates, inertia weight, cognitive vs social components, and fitness landscape exploration for swarms."
triggers: ["kennedy", "eberhart", "particle-swarm", "pso-optimization", "swarm-intelligence", "fitness-landscape", "velocity-update"]
---

# agentic-kennedy-swarm-intelligence
> Based on **Swarm Intelligence - James Kennedy & Russell Eberhart**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **PSO Velocity Update: v_i(t+1) = w*v_i(t) + c_1*r_1*(pbest_i - x_i(t)) + c_2*r_2*(gbest - x_i(t)).**
2. **Inertia Weight w: Balances exploration (high w) vs exploitation (low w), typically decayed dynamically over iterations.**
3. **Socio-Cognitive Duality: Individual cognitive memory (pbest) balances social consensus (gbest) to escape local optima in solution space.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Optimize agent hyper-parameters, prompt candidates, or architecture topology using Particle Swarm dynamics. Balance autonomous exploration with flock-wide best solution exploitation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Premature convergence to suboptimal local minima due to excessive social attraction weight.**
- **Static velocity parameters causing particles to oscillate wildly across fitness boundaries.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kennedy"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
