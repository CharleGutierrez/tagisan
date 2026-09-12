---
name: arch-rosenthal-chaos-engineering
description: "Hypothesis-driven chaos experiments: Steady-state verification, automated fault injection, blast radius containment, GameDays, and catastrophic failure mitigation."
triggers: ["chaos-engineering", "fault-injection", "steady-state-verification", "blast-radius", "chaos-monkey", "game-days", "resilience-testing"]
---

# arch-rosenthal-chaos-engineering
> Based on **Chaos Engineering: System Resiliency in Practice - Casey Rosenthal & Nora Jones**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Formulate chaos experiments with a verifiable hypothesis based on normal steady-state behavior (e.g. 99% of requests succeed with <200ms latency).**
2. **ALWAYS: Limit the blast radius of chaos fault injections to a canary percentage (e.g. 5% of traffic) with automated emergency abort switches.**
3. **NEVER: Run chaos experiments in production without real-time observability telemetry validating steady-state thresholds.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define steady-state metrics. Conduct controlled fault-injection tests (kill instances, sever network links, inject 500ms latency). Verify graceful degradation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Injecting massive cluster-wide failures without automated abort triggers.**
- **Relying on unit tests alone while never verifying behavior during network partition.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-rosenthal-chaos-engineering"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
