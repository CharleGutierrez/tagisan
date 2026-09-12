---
name: arch-kleppmann-distributed-data
description: "Foundational distributed data systems: Unreliable networks, clock skew, linearizability vs. serializability, hybrid logical clocks, consensus limits, and dual-write mitigations."
triggers: ["kleppmann", "distributed-data", "linearizability", "serializability", "dual-write", "hybrid-logical-clocks", "unreliable-network", "two-phase-commit-fallacy"]
---

# arch-kleppmann-distributed-data
> Based on **Designing Data-Intensive Applications - Martin Kleppmann**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Reject system wall-clock (NTP) for transaction ordering; enforce monotonic sequencer, Lamport timestamps, or Hybrid Logical Clocks (HLC).**
2. **NEVER: Execute dual-writes to database and message broker without a Transactional Outbox or Change Data Capture (CDC).**
3. **STRICT_REJECT: Distributed transactions assuming network synchrony or instantaneous message delivery.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define data consistency models (Linearizable, Sequential, Causal, Read-After-Write). Enforce Transactional Outbox for all broker events. Eliminate reliance on NTP timestamps.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using local system time for distributed event ordering.**
- **Writing to database then publishing to Kafka without transactional outbox.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-kleppmann-distributed-data"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
