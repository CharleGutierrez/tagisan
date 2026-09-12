---
name: qual-skelton-team-topologies
description: "Sociotechnical system design: Conway's Law in practice, managing team cognitive load, four team types (Stream-Aligned, Enabling, Complicated-Subsystem, Platform)."
triggers: ["skelton-pais", "team-topologies", "conways-law", "team-cognitive-load", "stream-aligned", "platform-team", "fast-flow", "fracture-planes"]
---

# qual-skelton-team-topologies
> Based on **Team Topologies: Organizing Business and Technology Teams for Fast Flow - Matthew Skelton & Manuel Pais**

## 1. Core Psychological Foundations & Formal Invariants

1. **ALWAYS: Align software architecture and service boundaries strictly with individual team cognitive load limits.**
2. **ALWAYS: Provide self-service internal developer platforms (IDPs) that reduce cognitive load for stream-aligned teams.**
3. **NEVER: Create sprawling shared services or monoliths whose blast radius exceeds the cognitive comprehension of a single team.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Decompose system modules along cognitive fracture planes, matching service boundaries to autonomous team cognitive capacities.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assigning a 50-service microservice mesh to a single 3-person team.**
- **Monolithic codebases requiring cross-team approval for single-line changes.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "qual-skelton-team-topologies"

# Execute automated qualitative & behavioral audit
cargo test --test qual_skills_brutal_tests
```
