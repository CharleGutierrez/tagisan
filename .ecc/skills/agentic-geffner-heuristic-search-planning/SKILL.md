---
name: agentic-geffner-heuristic-search-planning
description: "Delete-relaxation heuristics, landmark heuristics, state-space reduction, and satisficing vs optimal search in complex task graphs."
triggers: ["geffner", "bonet", "heuristic-search-planning", "delete-relaxation", "landmark-heuristics", "satisficing-planning"]
---

# agentic-geffner-heuristic-search-planning
> Based on **A Concise Introduction to Models and Methods for Automated Planning - Hector Geffner & Blai Bonet**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Delete-Relaxation Heuristic (h+): Ignoring negative effects of actions yields an admissible relaxation of the true distance to the goal.**
2. **Fact & Action Landmarks: Subgoals that must be true at some point in every valid plan, providing mandatory stepping stones.**
3. **Satisficing Search: Trading bounded suboptimality for polynomial-time planning speed using Enforced Hill-Climbing or Greedy Best-First Search.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Identify architectural landmarks (e.g. database schema migrated, interface trait compiled) before writing implementation code. Use satisficing heuristics for rapid iteration.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Exhaustive brute-force search over massive solution spaces when satisficing greedy search suffices.**
- **Abandoning landmark milestones, leading to circular refactoring with no measurable progress.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "geffner"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
