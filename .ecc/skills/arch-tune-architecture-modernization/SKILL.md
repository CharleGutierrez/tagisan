---
name: arch-tune-architecture-modernization
description: "Strategic domain modernization: Wardley Mapping, Bounded Context Canvas, Core vs Supporting Domains, Anti-Corruption Layers (ACL), and socio-technical team topologies."
triggers: ["tune-modernization", "bounded-context-canvas", "anti-corruption-layer", "wardley-mapping", "core-domain", "legacy-migration", "socio-technical"]
---

# arch-tune-architecture-modernization
> Based on **Architecture Modernization - Nick Tune**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Modernized bounded contexts must interface with legacy services through an explicit Anti-Corruption Layer (ACL) with DTO translation.**
2. **ALWAYS: Prioritize engineering effort and custom code on Core Strategic Domains; outsource or adopt off-the-shelf software for Generic/Commodity domains.**
3. **NEVER: Allow legacy domain models to contaminate greenfield bounded context schemas.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Map system domains into Core, Supporting, and Generic using Wardley Mapping. Enforce Anti-Corruption Layers at all legacy migration boundaries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Directly exposing legacy database tables to new microservices without translation.**
- **Treating all services as equally important without strategic domain triage.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-tune-architecture-modernization"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
