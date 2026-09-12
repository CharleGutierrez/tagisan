---
name: arch-nygard-release-it
description: "Production resilience patterns: Circuit Breakers, Bulkheads, Timeouts, Shed Load, Fail Fast, and antipatterns like Cascading Failures, Dogpiling, and Unbounded Pools."
triggers: ["nygard-release-it", "circuit-breaker-pattern", "bulkhead-pattern", "shed-load", "cascading-failure", "fail-fast", "production-resilience"]
---

# arch-nygard-release-it
> Based on **Release It!: Design and Deploy Production-Ready Software - Michael T. Nygard**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Isolate external downstream integration points with dedicated Bulkheads (separate thread/connection pools) so one failing dependency cannot starve the host.**
2. **ALWAYS: Wrap network calls in Circuit Breakers that trip to OPEN state upon consecutive error thresholds.**
3. **NEVER: Configure unbounded queues or connection pools for external services.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Equip every external API client with a Circuit Breaker, a Bulkhead connection pool, and strict socket timeouts. Fail fast when resource saturation exceeds safe thresholds.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Shared connection pool where one slow third-party API exhausts all server threads.**
- **Missing socket timeouts leading to hung connections indefinitely.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-nygard-release-it"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
