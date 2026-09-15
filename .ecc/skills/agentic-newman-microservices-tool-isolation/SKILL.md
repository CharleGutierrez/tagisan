---
name: agentic-newman-microservices-tool-isolation
description: "Loose coupling, high cohesion, bounded contexts, API versioning, canary deployments, circuit breakers, and sandboxed tool isolation."
triggers: ["newman", "building-microservices", "service-isolation", "bounded-contexts", "circuit-breakers", "canary-deployment"]
---

# agentic-newman-microservices-tool-isolation
> Based on **Building Microservices: Designing Fine-Grained Systems - Sam Newman**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **High Cohesion & Loose Coupling: Code that changes together stays together; services know as little as possible about each other's internals.**
2. **Circuit Breaker Pattern: Automatically tripping open to stop calling a failing dependency, returning fast fallbacks rather than cascading failures.**
3. **Backwards-Compatible API Versioning: Tolerant reader pattern ensuring schema additions do not break existing downstream service clients.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Isolate agent tool execution within microservice boundaries with strict circuit breakers and timeouts. Protect production backends from cascading agent retries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Cascading failures where a single failing agent tool crashes the entire orchestrator.**
- **Breaking API contract changes that break downstream client agents.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "newman"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
