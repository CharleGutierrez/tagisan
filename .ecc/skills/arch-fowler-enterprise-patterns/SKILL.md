---
name: arch-fowler-enterprise-patterns
description: "Canonical enterprise application architecture: Unit of Work, Identity Map, Repository, Data Mapper, Domain Model, Service Layer, and Optimistic Offline Locking."
triggers: ["fowler-poeaa", "unit-of-work", "identity-map", "data-mapper", "domain-model", "optimistic-offline-lock", "enterprise-architecture"]
---

# arch-fowler-enterprise-patterns
> Based on **Patterns of Enterprise Application Architecture (PoEAA) - Martin Fowler**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Domain entities must remain persistence-ignorant; all database interaction is mediated by Repository and Data Mapper boundaries.**
2. **ALWAYS: Multi-entity updates in a single business request must be tracked in a Unit of Work to prevent partial database flushes.**
3. **ALWAYS: Prevent lost concurrent updates using Optimistic Offline Locking (version column check).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Domain Model with Unit of Work and Identity Map. Encapsulate business rules in entities, not database triggers or anemic DTOs. Enforce version-based concurrency checks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Anemic domain models with business logic scattered across controllers and SQL queries.**
- **Overwriting concurrent updates without checking entity version stamps.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-fowler-enterprise-patterns"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
