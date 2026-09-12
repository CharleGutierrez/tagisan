---
name: ba-assumption-mapping-experimentation
description: "Assumption Mapping & Experiment Design: The 2x2 Importance vs Evidence grid, riskiest assumption tests (RAT), prototype fidelity ladders, and experiment loops."
triggers: ["assumption-mapping-experimentation", "assumption-mapping", "david-bland", "alex-osterwalder", "testing-business-ideas", "rat-riskiest-assumption"]
---

# ba-assumption-mapping-experimentation
> Based on **Testing Business Ideas - David J. Bland & Alex Osterwalder**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Riskiest Assumption First (RAT): Only invest engineering resources in testing assumptions categorized in the top-right quadrant (High Importance, Low Evidence).**
2. **Falsifiable Hypotheses: Experiments must define a pass/fail threshold before data collection starts.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Plot assumptions on a 2x2 grid (Importance vs Evidence). Design quick spikes or prototypes for High Importance / Low Evidence assumptions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Testing low-risk assumptions because they are easy to measure.**
- **Moving forward with development after an experiment fails its pre-set benchmark.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "assumption-mapping-experimentation"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
