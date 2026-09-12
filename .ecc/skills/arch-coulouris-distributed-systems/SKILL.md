---
name: arch-coulouris-distributed-systems
description: "Classical distributed systems theory: RPC failure semantics (at-least-once, at-most-once), vector clocks, causal ordering, distributed mutual exclusion, and Byzantine fault models."
triggers: ["coulouris", "distributed-systems-theory", "vector-clocks", "causal-ordering", "rpc-semantics", "byzantine-fault", "distributed-mutual-exclusion"]
---

# arch-coulouris-distributed-systems
> Based on **Distributed Systems: Concepts and Design - George Coulouris, Jean Dollimore, Tim Kindberg, Gordon Blair**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Design all distributed network mutations for at-least-once transport with explicit idempotent receiver deduplication.**
2. **ALWAYS: Track causal precedence of concurrent distributed operations using Vector Clocks when multi-master replication is enabled.**
3. **NEVER: Assume exactly-once delivery across uncoordinated network boundaries without end-to-end idempotency tokens.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Specify RPC failure modes (crash-stop vs crash-recovery). Attach unique transaction/mutation UUIDs to enable idempotent retries. Implement vector clocks for concurrent updates.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming RPC calls never fail or time out.**
- **Treating distributed network calls as identical to in-memory function invocations.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-coulouris-distributed-systems"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
