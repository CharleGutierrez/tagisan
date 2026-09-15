---
name: agentic-kim-phoenix-project-flow-theory
description: "The Three Ways (Flow, Feedback, Continual Learning), Theory of Constraints (Goldratt), work-in-progress (WIP) limits, and bottleneck management."
triggers: ["kim", "phoenix-project", "three-ways", "theory-of-constraints", "wip-limits", "bottleneck-management", "devops-flow"]
---

# agentic-kim-phoenix-project-flow-theory
> Based on **The Phoenix Project - Gene Kim, Kevin Behr & George Spafford**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **The First Way (Principles of Flow): Accelerate the flow of work from Development to Operations; reduce batch sizes and WIP.**
2. **The Second Way (Principles of Feedback): Create fast, reciprocal feedback loops from right to left; amplify feedback to prevent recurrence of errors.**
3. **The Third Way (Continual Learning): Foster a culture of experimentation, calculated risk-taking, and learning from failure.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Identify the primary bottleneck in the software pipeline (e.g. slow tests, manual deploys) and subordinate all agent activities to resolving it.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Optimizing non-bottleneck stages, creating inventory piles without increasing throughput.**
- **Ignoring operational feedback and continuing to push code into broken environments.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kim"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
