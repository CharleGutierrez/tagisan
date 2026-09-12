---
name: arch-newman-building-microservices
description: "Microservices architectural foundations: Independent deployability, service decomposition by business capability, consumer-driven contracts, and avoiding shared databases."
triggers: ["sam-newman", "building-microservices", "independent-deployability", "service-decomposition", "consumer-driven-contracts", "microservices-foundations"]
---

# arch-newman-building-microservices
> Based on **Building Microservices (2nd Edition) - Sam Newman**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Every microservice must own its private datastore; direct cross-service database access is strictly prohibited.**
2. **ALWAYS: Maintain independent deployability: modifying one service must never mandate simultaneous deployment of another service.**
3. **NEVER: Share database tables, schemas, or foreign keys across microservice boundaries.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Decompose systems along business capability boundaries. Enforce API contracts (OpenAPI/Protobuf) with backward-compatibility checks. Isolate datastores per service.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Two microservices querying the same SQL database tables directly.**
- **Lock-step deployments where Service A v2 requires Service B v2 to be deployed simultaneously.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-newman-building-microservices"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
