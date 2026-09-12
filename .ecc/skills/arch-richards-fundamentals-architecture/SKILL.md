---
name: arch-richards-fundamentals-architecture
description: "Architectural principles: Architectural characteristics (-ilities), component cohesion, connascence, architectural styles, and trade-off evaluation."
triggers: ["fundamentals-software-architecture", "architectural-characteristics", "connascence", "component-cohesion", "architecture-styles", "tradeoff-analysis"]
---

# arch-richards-fundamentals-architecture
> Based on **Fundamentals of Software Architecture - Mark Richards & Neal Ford**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Analyze and minimize Connascence of Execution, Timing, and Algorithm between collaborating components.**
2. **ALWAYS: Explicitly document the top 3 architectural characteristics (e.g. Scalability, Security, Maintainability) driving design decisions.**
3. **NEVER: Prioritize non-functional characteristics that conflict with core business drivers without documented consensus.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Measure coupling via static and dynamic connascence. Select architectural styles (Microkernel, Event-Driven, Space-Based, Microservices) based on explicit characteristic drivers.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Prematurely adopting microservices when the application requirements favor a modular monolith.**
- **Ignoring timing connascence where services assume identical network speeds.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-richards-fundamentals-architecture"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
