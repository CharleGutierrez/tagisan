---
name: agentic-tanenbaum-distributed-systems
description: "RPC protocols, message-oriented middleware, distributed naming, synchronization, fault tolerance, and process migration."
triggers: ["tanenbaum", "van-steen", "distributed-systems", "remote-procedure-call", "rpc-protocols", "fault-tolerance", "distributed-naming"]
---

# agentic-tanenbaum-distributed-systems
> Based on **Distributed Systems: Principles and Paradigms - Andrew S. Tanenbaum & Maarten van Steen**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Fallacies of Distributed Computing: The network is reliable; latency is zero; bandwidth is infinite; the network is secure; topology doesn't change.**
2. **Idempotent Remote Procedure Calls: Remote operations must be idempotent so retransmissions do not cause duplicate side effects: f(f(x)) == f(x).**
3. **Heartbeat & Lease Heartbeats: Detecting node failure using periodic heartbeats with bounded timeout thresholds.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Wrap all inter-agent RPCs in idempotent envelopes with unique idempotency keys. Implement exponential backoff and jitter on network retries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Treating remote API calls as synchronous local method calls, ignoring latency and network partitions.**
- **Non-idempotent endpoints that double-charge or create duplicate records on network retry.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "tanenbaum"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
