---
name: arch-ozsu-distributed-databases
description: "Distributed database architecture: Horizontal and vertical fragmentation, distributed query optimization, 2-phase commit (2PC), distributed deadlock detection, and data replication."
triggers: ["ozsu", "distributed-databases", "database-fragmentation", "distributed-query-optimization", "two-phase-commit", "distributed-deadlock", "shard-key"]
---

# arch-ozsu-distributed-databases
> Based on **Principles of Distributed Database Systems - M. Tamer Özsu & Patrick Valduriez**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Partition data along high-cardinality shard keys aligned with primary query predicate to enable single-partition query routing.**
2. **NEVER: Execute unindexed multi-shard cross-partition distributed joins across cluster nodes in real-time OLTP requests.**
3. **AUDIT: Two-phase commit (2PC) coordinator blocking timeouts and implement heuristic transaction commit/rollback resolution.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define horizontal/vertical table partitioning schemes. Ensure queries specify the shard/tenant key to avoid broadcast queries. Document distributed 2PC coordinator failure handlers.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Performing distributed cross-partition joins in latency-sensitive user paths.**
- **Leaving distributed 2PC participants blocked indefinitely upon coordinator crash.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-ozsu-distributed-databases"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
