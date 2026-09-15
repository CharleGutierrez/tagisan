---
name: agentic-shoham-multiagent-foundations
description: "Game-theoretic equilibria, Nash equilibrium, mechanism design, distributed constraint satisfaction (DisCSP), and social choice rules for multi-agent resource allocation."
triggers: ["shoham", "leyton-brown", "game-theory", "nash-equilibrium", "discsp", "mechanism-design", "social-choice", "distributed-constraint"]
---

# agentic-shoham-multiagent-foundations
> Based on **Multiagent Systems: Algorithmic, Game-Theoretic, and Logical Foundations - Yoav Shoham & Kevin Leyton-Brown**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Nash Equilibrium: Strategy profile where no agent has incentive to unilaterally deviate given others' strategies: u_i(s_i^*, s_{-i}^*) >= u_i(s_i, s_{-i}^*).**
2. **Distributed Constraint Satisfaction (DisCSP): Agents solve local constraints while communicating state via Asynchronous Backtracking (ABT) or Asynchronous Weak-Commitment (AWC).**
3. **Vickrey-Clarke-Groves (VCG) Mechanism: Truthful dominant-strategy mechanism design aligning private incentives with global system utility.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure multi-agent resource allocation and arbitration using game-theoretic mechanism design. Enforce incentive compatibility so truthful reporting is the dominant strategy. Solve distributed resource conflicts via DisCSP constraint propagation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming cooperative agents without aligning individual utility functions, inviting tragedy of the commons.**
- **Naive priority-based arbitration prone to starvation and cyclic bidding wars.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "shoham"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
