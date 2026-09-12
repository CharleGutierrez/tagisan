---
name: ba-meadows-systems-thinking-leverage
description: "Systems Thinking & Dynamic Stocks and Flows: Stocks, inflows, outflows, balancing/reinforcing feedback loops, system delays, and the 12 leverage points."
triggers: ["meadows-systems-thinking-leverage", "systems-thinking", "donella-meadows", "stocks-and-flows", "feedback-loops", "leverage-points", "system-delays"]
---

# ba-meadows-systems-thinking-leverage
> Based on **Thinking in Systems: A Primer - Donella H. Meadows**

## 1. Core Mathematical Foundations & Formal Analysis Invariants

1. **Stock and Flow Conservation: dS/dt = Inflow(t) - Outflow(t). A stock can only change through its inflows and outflows.**
2. **Leverage Hierarchy: Parameter changes have the lowest leverage; shifting system goals, rules, and mindsets has the highest systemic leverage.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Analyze problem domains as dynamic systems: Define Stocks (accumulations), Inflows, Outflows, Delays, and Feedback Loops.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Treating symptoms by modifying surface parameters while ignoring broken system feedback loops.**
- **Ignoring delays and over-correcting system controls.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "meadows-systems-thinking-leverage"

# Execute automated functional audit
cargo test --test ba_skills_brutal_tests
```
