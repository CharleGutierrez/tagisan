---
name: agentic-csikszentmihalyi-flow-state
description: "Challenge-skill equilibrium, clear proximal goals, unambiguous feedback, deep immersion, distortion of temporal perception, and cognitive ergonomics for developers."
triggers: ["csikszentmihalyi", "flow-state", "challenge-skill-balance", "unambiguous-feedback", "cognitive-ergonomics", "developer-immersion"]
---

# agentic-csikszentmihalyi-flow-state
> Based on **Flow: The Psychology of Optimal Experience - Mihaly Csikszentmihalyi**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Flow Channel Condition: Task challenge C and operator skill S must remain balanced: C approx S. If C >> S -> anxiety; if S >> C -> boredom.**
2. **Immediate Feedback Invariant: System responses must arrive within sub-second thresholds to prevent breaking the developer's working memory.**
3. **Clear Proximal Subgoals: Deconstruct ambiguous epic goals into clear, incremental milestones achievable in minutes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Preserve developer flow state at all costs. Provide immediate, deterministic feedback for every code edit and break daunting tasks into bite-sized achievable steps.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Unresponsive tools or long silent pauses without progress telemetry.**
- **Presenting overwhelming 50-step plans that induce cognitive overload and anxiety.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "csikszentmihalyi"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
