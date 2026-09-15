---
name: agentic-murray-consensus-cooperation
description: "Graph Laplacians, algebraic connectivity, consensus protocols, agreement algorithms under directed delay topologies, and distributed swarm formation control."
triggers: ["murray", "olfati-saber", "consensus-cooperation", "graph-laplacian", "algebraic-connectivity", "agreement-protocol", "swarm-consensus"]
---

# agentic-murray-consensus-cooperation
> Based on **Consensus and Cooperation in Networked Multi-Agent Systems - Reza Olfati-Saber, J. Alex Fax & Richard M. Murray**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Continuous-time Consensus: dx_i/dt = - sum_j a_ij * (x_i - x_j) = - [L * x]_i, where L = D - A is the graph Laplacian matrix.**
2. **Algebraic Connectivity lambda_2(L): Consensus is achieved asymptotically if and only if the communication network graph has a spanning tree (lambda_2 > 0).**
3. **Delay Robustness: Nyquist criteria bounds the maximum allowable communication delay tau < pi / (2 * lambda_max(L)) for stability.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce distributed state consensus across agent swarms using Laplacian feedback. Verify algebraic connectivity of the communication graph to guarantee synchronization before execution.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Partitioned communication graphs where disconnected agent clusters diverge into incompatible states.**
- **Network message latency exceeding theoretical stability bounds, triggering oscillation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "murray"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
