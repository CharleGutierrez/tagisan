---
name: agentic-kleppmann-distributed-consistency
description: "ACID vs BASE, linearizability, eventual consistency, leader-follower replication, partitioning, distributed transactions, and event sourcing."
triggers: ["kleppmann", "ddia", "data-intensive-applications", "linearizability", "eventual-consistency", "event-sourcing", "distributed-consensus"]
---

# agentic-kleppmann-distributed-consistency
> Based on **Designing Data-Intensive Applications - Martin Kleppmann**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **CAP Theorem & Trade-offs: In the presence of a network partition (P), a distributed system must choose between Consistency (C) or Availability (A).**
2. **Linearizability: All operations appear to execute atomically at a single instant in time between their invocation and response.**
3. **Two-Phase Commit (2PC) Vulnerability: Coordinator failure during the prepare/commit window blocks participants indefinitely.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design agent persistent stores with clear consistency guarantees. Use event-sourced logs for auditability and idempotent operations for safe retries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming network calls never fail or time out, omitting retry backoff and circuit breakers.**
- **Relying on distributed locks without fencing tokens, causing split-brain storage writes.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "kleppmann"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
