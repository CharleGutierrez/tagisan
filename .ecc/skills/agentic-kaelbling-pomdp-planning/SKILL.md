---
name: agentic-kaelbling-pomdp-planning
description: "Belief state updates, observation uncertainty, policy trees, value iteration in continuous probability simplex, and active sensing in partially observable environments."
triggers: ["kaelbling", "littman", "pomdp", "partially-observable", "belief-state", "observation-probability", "active-sensing"]
---

# agentic-kaelbling-pomdp-planning
> Based on **Planning and Acting in Partially Observable Stochastic Domains (POMDPs) - Leslie Pack Kaelbling, Michael L. Littman & Anthony R. Cassandra**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Belief State Update: b'(s') = P(o | s', a) * sum_s P(s' | s, a) * b(s) / P(o | b, a).**
2. **POMDP Tuple: Defined as (S, A, T, R, Omega, O, gamma), where Omega is observation space and O is observation probability.**
3. **Active Information Gathering: Selecting actions whose primary utility is reducing epistemic entropy over system state (e.g. running diagnostics).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model unseen third-party API states and legacy systems as POMDPs. Execute active probe queries (diagnostics, logs, dry-runs) to collapse belief state uncertainty before mutating code.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming complete observability of production systems, leading to blind overwrites of hidden state.**
- **Ignoring observation noise (flaky tests) and misclassifying system health.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kaelbling"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
