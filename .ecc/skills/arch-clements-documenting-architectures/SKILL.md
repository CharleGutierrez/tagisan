---
name: arch-clements-documenting-architectures
description: "Formal architecture documentation: 4+1 View Model, Module Views, Component-and-Connector (C&C) Views, Allocation Views, and interface specifications."
triggers: ["documenting-architectures", "views-and-beyond", "component-and-connector", "module-views", "allocation-views", "4-plus-1-views", "architectural-views"]
---

# arch-clements-documenting-architectures
> Based on **Documenting Software Architectures: Views and Beyond - Paul Clements et al.**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Distinguish Module Views (static code, classes, layers) from Component-and-Connector Views (runtime processes, threads, sockets, pipes).**
2. **ALWAYS: Document every architectural interface with syntax, semantic invariants, error states, and quality attribute bounds.**
3. **NEVER: Present an architectural diagram where boxes and arrows have ambiguous or mixed semantics.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Document architectures across the 3 fundamental viewtypes: Module Views (structure), Component-and-Connector Views (runtime), and Allocation Views (deployment/hardware).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Mixing static code dependencies and dynamic network connections in the same diagram.**
- **Diagrams with unlabeled arrows and undefined box semantics.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-clements-documenting-architectures"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
