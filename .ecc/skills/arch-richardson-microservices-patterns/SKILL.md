---
name: arch-richardson-microservices-patterns
description: "Transactional distributed patterns: Saga pattern (Orchestration vs Choreography), CQRS, Event Sourcing, API Gateway, and Distributed Queries."
triggers: ["chris-richardson", "microservices-patterns", "saga-pattern", "orchestrated-saga", "choreographed-saga", "cqrs", "event-sourcing", "api-gateway"]
---

# arch-richardson-microservices-patterns
> Based on **Microservices Patterns - Chris Richardson**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Multi-service distributed transactions must be orchestrated via Sagas with explicit compensating transactions for rollback on failure.**
2. **ALWAYS: Materialize cross-service read queries using Command Query Responsibility Segregation (CQRS) views built from domain event streams.**
3. **NEVER: Use Two-Phase Commit (2PC) or XA transactions across distributed microservices.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Saga orchestrator for multi-step distributed operations (e.g. CreateOrder -> ReserveCredit -> Fulfill). Define compensation actions for every step. Use CQRS for read views.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using distributed 2PC locks across microservices, creating distributed deadlocks.**
- **Direct synchronous fan-out REST queries to 10 services to assemble a single read view.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-richardson-microservices-patterns"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
