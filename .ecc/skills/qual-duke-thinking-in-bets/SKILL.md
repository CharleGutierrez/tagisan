---
name: qual-duke-thinking-in-bets
description: "Probabilistic decision-making under uncertainty: Decoupling decision quality from outcomes ('resulting'), combating hindsight bias, and scenario forecasting."
triggers: ["annie-duke", "thinking-in-bets", "resulting-fallacy", "probabilistic-decisions", "hindsight-bias", "decision-audit", "uncertainty"]
---

# qual-duke-thinking-in-bets
> Based on **Thinking in Bets: Making Smarter Decisions When You Don't Have All the Facts - Annie Duke**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Record decision quality, assumptions, and confidence scores at the moment of execution, decoupled from runtime outcomes.**
2. **ALWAYS: Present predictions and recommendations to users with explicit confidence intervals and probabilistic uncertainty bounds.**
3. **NEVER: Commit the 'resulting' fallacy by punishing users or rewriting rules simply because a high-probability decision encountered a low-probability variance.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Incorporate explicit probabilistic modeling and confidence calibration into system alerts and decision logs, preventing hindsight bias.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Displaying binary deterministic assertions for machine learning predictions.**
- **Blaming operators for executing sound probabilistic protocols when rare variance occurs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-duke-thinking-in-bets"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
