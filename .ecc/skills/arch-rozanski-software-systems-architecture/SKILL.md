---
name: arch-rozanski-software-systems-architecture
description: "Viewpoints and Perspectives framework: Functional, Information, Concurrency, Development, Deployment, Operational viewpoints; and Security, Performance, Availability perspectives."
triggers: ["rozanski-woods", "viewpoints-and-perspectives", "concurrency-viewpoint", "information-viewpoint", "availability-perspective", "architectural-perspectives"]
---

# arch-rozanski-software-systems-architecture
> Based on **Software Systems Architecture - Nick Rozanski & Eóin Woods**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Review every architectural design against cross-cutting Perspectives: Security, Performance, Availability, and Evolution.**
2. **ALWAYS: Validate the Concurrency Viewpoint: identify all concurrent execution threads, shared resources, and synchronization primitives to prevent deadlocks.**
3. **NEVER: Complete an architectural spec without detailing the Operational and Deployment Viewpoints.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Evaluate architecture across core viewpoints (Functional, Information, Concurrency, Deployment) and apply cross-cutting perspectives (Security, Performance, Availability).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Designing systems without a concurrency viewpoint, leading to race conditions in production.**
- **Treating operational deployment as an afterthought delegated entirely to operations.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-rozanski-software-systems-architecture"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
