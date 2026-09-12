---
name: arch-ford-software-architecture-hard-parts
description: "Modern distributed architecture trade-offs: Service granularity, distributed data decomposition, transactional sagas, contract management, and coupling analysis."
triggers: ["architecture-hard-parts", "architectural-tradeoffs", "granularity-disintegrators", "saga-orchestration", "data-decomposition", "connascence"]
---

# arch-ford-software-architecture-hard-parts
> Based on **Software Architecture: The Hard Parts - Neal Ford, Mark Richards, Pramod Sadalage, Zhamak Dehghani**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Every architectural proposal must document explicit trade-offs across consistency, latency, elasticity, and operational complexity.**
2. **ALWAYS: Decompose monolith services only when explicit disintegrators (differing scalability, team autonomy, security boundaries) justify distributed overhead.**
3. **NEVER: Split services without decomposing their underlying database schemas.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Apply architectural disintegrator/integrator drivers to justify service boundaries. Avoid distributed transactions; select Choreographed or Orchestrated Sagas with compensating actions.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Splitting a monolith into microservices while leaving a shared monolithic database.**
- **Adopting distributed microservices for small applications without team/scale justifications.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-ford-software-architecture-hard-parts"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
