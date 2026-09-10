---
name: domain-driven-design
description: "Domain-Driven Design (Eric Evans & Vlad Khononov): Ubiquitous Language, Bounded Contexts, Aggregate Roots, Entities, Value Objects, Domain Events, and Anti-Corruption Layers for enterprise AI development."
triggers: ["domain driven design", "ddd", "bounded context", "aggregate root", "value object", "ubiquitous language", "domain events", "eric evans", "anti corruption layer", "context mapping"]
---

# Domain-Driven Design (Eric Evans & Vlad Khononov)

This skill guides the agent in modeling complex business domains using the strategic and tactical patterns of Domain-Driven Design (DDD).

## 1. Strategic Design: Boundaries & Ubiquitous Language
1. **Ubiquitous Language**:
   - Establish a shared, strictly defined domain lexicon between domain experts and software code.
   - Refuse technical slang or ambiguous synonyms (e.g., if the court calls it a `DocketNumber`, never call it `case_code` or `file_id` in code).
2. **Bounded Contexts**:
   - Partition large systems into autonomous linguistic and model boundaries.
   - Example: In the `DocketingContext`, a `Case` models parties, raffles, and court branch assignments. In the `LegalFeeContext`, the same physical folder is modeled as a `FeeAssessmentAccount`. Never merge distinct contexts into a bloated God Object.
3. **Context Mapping & Anti-Corruption Layers (ACL)**:
   - When integrating with legacy systems or third-party APIs, always insert an **Anti-Corruption Layer (ACL)** to translate foreign data models into pure internal domain types.

## 2. Tactical Design: Aggregates & Value Objects
1. **Value Objects**:
   - Immutable objects with structural equality (no identity).
   - Self-validating upon instantiation (e.g., `Money`, `DocketNumber`, `EmailAddress`).
   - If an attribute has business validation rules, it MUST be a Value Object, not a raw primitive string or float.
2. **Entities**:
   - Objects possessing a unique, enduring Identity across their lifecycle.
   - Mutate internal state only through explicit domain methods, never raw public setters.
3. **Aggregates & Aggregate Roots (The Consistency Boundary)**:
   - Group related Entities and Value Objects into an Aggregate under a single **Aggregate Root**.
   - External callers may ONLY reference and interact with the Aggregate Root.
   - A database transaction must update only ONE aggregate root at a time to ensure ACID consistency without distributed lock contention.
4. **Domain Events**:
   - Emit immutable past-tense event records whenever state changes (`CaseRaffled`, `FeePaid`, `SummonsIssued`).
   - Decouple side effects (emails, audit logs, external sync) via asynchronous domain event handlers.

## 3. Anti-Patterns to Prevent
- **Anemic Domain Model**: Domain entities that are mere bags of getters/setters with business logic scattered across procedural services. Put business invariants inside the Aggregate!
- **Leaky Database Concerns**: Keep ORM annotations and SQL queries out of core domain entities.
