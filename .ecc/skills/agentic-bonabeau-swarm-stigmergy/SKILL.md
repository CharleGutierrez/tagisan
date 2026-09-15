---
name: agentic-bonabeau-swarm-stigmergy
description: "Stigmergic coordination, indirect communication via digital traces, division of labor, threshold response models, and collective ant trail algorithms."
triggers: ["bonabeau", "dorigo", "theraulaz", "stigmergy", "digital-pheromones", "ant-algorithms", "division-of-labor", "threshold-response"]
---

# agentic-bonabeau-swarm-stigmergy
> Based on **Swarm Intelligence: From Natural to Artificial Systems - Eric Bonabeau, Marco Dorigo & Guy Theraulaz**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Sematectonic Stigmergy: Environmental modification itself directs future actions (e.g. code artifacts and test results guide subsequent agent steps).**
2. **Sign-based Stigmergy: Specialized signals (digital pheromones, cache tags, event logs) guide collective agent routing.**
3. **Response Threshold Model: Agent i undertakes task j when stimulus s_j exceeds internal threshold theta_ij: P(perform) = s_j^2 / (s_j^2 + theta_ij^2).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Coordinate large agent swarms via stigmergic environmental traces rather than point-to-point RPCs. Let repository state, error logs, and build artifacts act as pheromones driving agent task pickup.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Flooding agents with synchronous point-to-point messages instead of reading environmental cues.**
- **Pheromone buildup without evaporation, leading to stale historical artifacts dominating decisions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "bonabeau"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
