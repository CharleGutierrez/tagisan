---
name: agentic-weiss-distributed-ai
description: "Distributed problem solving, multi-agent reinforcement learning, blackboard architectures, decentralized task allocation, and organizational structures for cooperative agents."
triggers: ["weiss", "distributed-ai", "blackboard-architecture", "distributed-problem-solving", "cooperative-agents", "coalition-formation"]
---

# agentic-weiss-distributed-ai
> Based on **Multiagent Systems: A Modern Approach to Distributed Artificial Intelligence - Gerhard Weiss**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Blackboard Architecture: Knowledge sources post hypotheses, partial solutions, and activations to a shared structured blackboard managed by a controller.**
2. **Multi-Agent Reinforcement Learning (MARL): Agents learn concurrently in non-stationary environments using value-function approximation or actor-critic policies.**
3. **Organizational Topologies: Hierarchy, flat market, federation, and holonic structures governing agent authority, scope, and communication channels.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design complex distributed problem solvers using a blackboard pattern for decoupled knowledge fusion. Define explicit organizational boundaries and escalation paths across specialist subagents.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Monolithic agent controllers creating single points of failure and throughput bottlenecks.**
- **Shared blackboard state without optimistic locking or transactional concurrency guards.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "weiss"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
