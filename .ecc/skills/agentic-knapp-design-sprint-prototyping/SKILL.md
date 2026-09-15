---
name: agentic-knapp-design-sprint-prototyping
description: "Timeboxed prototyping sprints, storyboarding, customer validation, facade prototypes, and rapid hypothesis testing without production code."
triggers: ["knapp", "zeratsky", "design-sprint", "facade-prototyping", "storyboard-validation", "timeboxed-sprints", "rapid-hypothesis"]
---

# agentic-knapp-design-sprint-prototyping
> Based on **Sprint: How to Solve Big Problems and Test New Ideas in Just Five Days - Jake Knapp, John Zeratsky & Braden Kowitz**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Facade Prototyping: Build the illusion of a finished system (using mock APIs and synthetic fixtures) to validate UX and business value in hours.**
2. **Timeboxing Discipline: Strict time limits force decision-making and prevent bikeshedding over non-critical edge cases.**
3. **Storyboard Mapping: Map critical user journeys end-to-end before implementing backend plumbing.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Rapidly scaffold interactive UI mockups and realistic API stubs so developers can test real product workflows before writing complex backend databases.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Spending days configuring backend databases before validating whether anyone wants the feature.**
- **Endless open-ended meetings without timeboxed prototype deliverables.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "knapp"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
