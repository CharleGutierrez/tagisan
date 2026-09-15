---
name: agentic-meadows-systems-thinking-feedback
description: "Stocks and flows, feedback loops (balancing and reinforcing), delays, system archetypes, and the 12 leverage points to intervene in a system."
triggers: ["meadows", "systems-thinking", "stocks-and-flows", "feedback-loops", "leverage-points", "system-archetypes", "delay-dynamics"]
---

# agentic-meadows-systems-thinking-feedback
> Based on **Thinking in Systems: A Primer - Donella H. Meadows**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Stock and Flow Dynamics: Stock is the memory of history (codebase size, tech debt, bug count); flow is the rate of change (commit rate, fix rate).**
2. **Balancing vs Reinforcing Loops: Reinforcing loops generate exponential growth or collapse; balancing loops resist change and enforce equilibrium.**
3. **High-Leverage Interventions: Changing goals, paradigms, and system rules produces orders of magnitude more impact than tweaking numerical parameters.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Analyze software architectures as living dynamic systems. Target high-leverage intervention points (e.g. automated CI gates, compiler types) rather than surface symptoms.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Treating systemic bugs as isolated one-off errors without addressing the reinforcing feedback loops that cause them.**
- **Ignoring delays in feedback loops, causing over-correction and catastrophic oscillations in codebase refactoring.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "meadows"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
