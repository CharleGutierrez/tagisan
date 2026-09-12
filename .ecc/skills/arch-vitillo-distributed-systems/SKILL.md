---
name: arch-vitillo-distributed-systems
description: "Pragmatic distributed engineering: Client-side timeouts, exponential backoff with full jitter, circuit breakers, gossip protocols, heartbeat leasing, and backpressure propagation."
triggers: ["vitillo", "understanding-distributed-systems", "circuit-breaker", "exponential-backoff", "jitter", "gossip-protocol", "heartbeating", "backpressure"]
---

# arch-vitillo-distributed-systems
> Based on **Understanding Distributed Systems - Roberto Vitillo**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Guard all downstream client requests with strict timeouts and exponential backoff with decorrelated full jitter.**
2. **ALWAYS: Trip circuit breakers when downstream error rate exceeds predefined threshold (e.g. 50% over 10s) to shed load and prevent cascading collapse.**
3. **NEVER: Retry immediately upon receiving 503 Service Unavailable or 429 Too Many Requests.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Equip all inter-service communication with circuit breakers, adaptive client timeouts, and bounded thread pools. Implement gossip protocols for node discovery.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Retrying failed RPC calls immediately in a tight loop, creating thundering herd outages.**
- **Unbounded inbound request queues without backpressure shedding.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-vitillo-distributed-systems"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
