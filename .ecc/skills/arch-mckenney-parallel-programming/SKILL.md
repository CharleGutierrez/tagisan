---
name: arch-mckenney-parallel-programming
description: "Advanced multi-core synchronization: Read-Copy Update (RCU), memory consistency models (Acquire-Release vs Sequential Consistency), cache coherence, and NUMA architecture."
triggers: ["mckenney-parallel", "rcu", "read-copy-update", "memory-ordering", "acquire-release", "numa-architecture", "cache-coherence"]
---

# arch-mckenney-parallel-programming
> Based on **Is Parallel Programming Hard, And, If So, What Can You Do About It? - Paul E. McKenney**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Read-heavy, write-rare shared data structures must use Read-Copy Update (RCU) or atomic pointer swap to achieve zero-synchronization reader throughput.**
2. **ALWAYS: Use explicit Acquire-Release memory ordering for atomic flags instead of defaulting to heavyweight Sequentially Consistent barriers.**
3. **NEVER: Cross NUMA node boundaries in latency-critical core loops without CPU thread pinning.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement RCU for concurrent reader scalability. Pin worker threads to NUMA cores. Use atomic acquire-release semantics for lock-free ring buffers.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using SeqCst atomic barriers across all variables, stalling the CPU instruction pipeline.**
- **Allowing thread migration across NUMA nodes during high-throughput computing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-mckenney-parallel-programming"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
