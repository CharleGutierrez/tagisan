---
name: agentic-reynolds-autonomous-agents
description: "Autonomous steering behaviors, separation, alignment, cohesion, obstacle avoidance, and decentralized crowd dynamics for autonomous software agents."
triggers: ["reynolds", "boids", "steering-behaviors", "separation-alignment-cohesion", "autonomous-flocking", "flocking-agents"]
---

# agentic-reynolds-autonomous-agents
> Based on **Flocks, Herds, and Schools: A Distributed Behavioral Model - Craig Reynolds**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Separation: Steer to avoid crowding local flockmates: F_sep = sum((x_i - x_j) / ||x_i - x_j||^2).**
2. **Alignment: Steer towards the average heading of local flockmates: F_align = mean(v_j) - v_i.**
3. **Cohesion: Steer to move toward the average position (center of mass) of local flockmates: F_coh = mean(x_j) - x_i.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use Reynolds flocking forces to coordinate concurrent code generation tasks. Keep agents separated in code scopes, aligned on architectural standards, and cohesive around the project vision.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Overlapping code edits causing git merge conflicts (separation failure).**
- **Incompatible architectural decisions across modules (alignment failure).**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "reynolds"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
