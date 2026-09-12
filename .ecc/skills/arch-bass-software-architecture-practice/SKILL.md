---
name: arch-bass-software-architecture-practice
description: "SEI architecture methodology: Quality Attribute Scenarios (QAW), Architecture Tradeoff Analysis Method (ATAM), Sensitivity Points, Trade-off Points, and Architectural Tactics."
triggers: ["sei-architecture", "atam", "quality-attribute-scenarios", "sensitivity-points", "tradeoff-points", "architectural-tactics", "software-architecture-practice"]
---

# arch-bass-software-architecture-practice
> Based on **Software Architecture in Practice (4th Edition) - Len Bass, Paul Clements, Rick Kazman**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Formulate non-functional quality attributes as concrete ATAM Scenarios: Stimulus, Source, Environment, Artifact, Response, and Response Measure.**
2. **ALWAYS: Identify and document Architectural Sensitivity Points (decisions critical to a specific attribute) and Trade-off Points (decisions affecting multiple conflicting attributes).**
3. **NEVER: Adopt architectural tactics without validating their trade-offs against system quality attributes.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Conduct ATAM evaluations for major design choices. Specify quality attributes with measurable responses (e.g. latency under 95% load). Map architectural tactics to attributes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Vague requirements like 'the system must be scalable' without stimulus/response metrics.**
- **Choosing architectural styles based on hype rather than ATAM quality trade-offs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-bass-software-architecture-practice"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
