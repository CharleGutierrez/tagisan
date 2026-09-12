---
name: arch-newman-monolith-to-microservices
description: "Evolutionary decomposition strategies: Strangler Fig pattern, database decomposition, Change Data Capture (CDC), branch by abstraction, and UI composition."
triggers: ["monolith-to-microservices", "strangler-fig", "database-decomposition", "change-data-capture", "branch-by-abstraction", "migration-patterns"]
---

# arch-newman-monolith-to-microservices
> Based on **Monolith to Microservices - Sam Newman**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Decompose monolithic applications incrementally using the Strangler Fig pattern behind an API Gateway/Reverse Proxy.**
2. **ALWAYS: Synchronize database separation asynchronously using Change Data Capture (CDC) or Transactional Outbox during the transition phase.**
3. **NEVER: Attempt a big-bang rewrite of a production monolithic system.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Strangler Fig reverse proxy to intercept and route legacy endpoints to new microservices. Use CDC for zero-downtime data synchronization.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Big-bang multi-year rewrites that fail before reaching production.**
- **Migrating application code to microservices while leaving the database monolithic.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-newman-monolith-to-microservices"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
