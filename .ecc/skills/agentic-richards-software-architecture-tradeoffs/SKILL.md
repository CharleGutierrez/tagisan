---
name: agentic-richards-software-architecture-tradeoffs
description: "Architectural characteristics (-ilities), modularity, component coupling, trade-off analysis, fitness functions, and architecture governance."
triggers: ["richards", "ford", "software-architecture", "architectural-tradeoffs", "fitness-functions", "component-coupling", "architecture-governance"]
---

# agentic-richards-software-architecture-tradeoffs
> Based on **Fundamentals of Software Architecture - Mark Richards & Neal Ford**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **First Law of Software Architecture: Everything in software architecture is a trade-off (performance vs simplicity, flexibility vs maintainability).**
2. **Automated Architectural Fitness Functions: Automated tests that execute in CI/CD to verify architecture characteristics (e.g. cycle detection, module coupling).**
3. **Connascence Metrics: Measuring the strength of coupling between components to minimize ripple effects of changes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Formulate architectural decisions as explicit trade-offs. Protect system structure by writing automated fitness functions that prevent circular dependencies.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Claiming an architectural choice has no downsides, ignoring hidden operational or latency costs.**
- **Allowing architectural degradation over time due to lack of automated fitness functions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "richards"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
