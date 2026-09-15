---
name: agentic-thrun-probabilistic-robotics
description: "Recursive Bayesian state estimation, Kalman filters, particle filters, SLAM (Simultaneous Localization and Mapping), and sensor fusion for autonomous agents."
triggers: ["thrun", "burgard", "fox", "probabilistic-robotics", "bayesian-estimation", "kalman-filter", "particle-filter", "slam"]
---

# agentic-thrun-probabilistic-robotics
> Based on **Probabilistic Robotics - Sebastian Thrun, Wolfram Burgard & Dieter Fox**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Bayes Filter Loop: Prediction: bel_bar(x_t) = int p(x_t | u_t, x_{t-1}) * bel(x_{t-1}) dx_{t-1}; Correction: bel(x_t) = eta * p(z_t | x_t) * bel_bar(x_t).**
2. **Particle Filtering: Representing arbitrary multimodal belief distributions via sets of weighted hypotheses (particles).**
3. **Sensor Fusion: Combining telemetry from unit tests, linter outputs, and runtime metrics to estimate system reliability.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain a Bayesian belief distribution over possible root causes during debugging. Update probabilities as compiler errors and diagnostic logs arrive.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Single-hypothesis fixation during debugging, ignoring contradictory diagnostic signals.**
- **Treating noisy runtime metrics as absolute ground truth without Bayesian filtering.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "thrun"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
