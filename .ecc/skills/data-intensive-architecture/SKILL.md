---
name: data-intensive-architecture
description: "Data-Intensive Applications Architecture (Martin Kleppmann): transactional isolation, ACID guarantees, idempotency keys, write-ahead logs, CQRS, eventual consistency, and distributed consensus."
triggers: ["data intensive", "kleppmann", "ddia", "distributed systems", "transactions", "acid", "idempotency", "write ahead log", "cqrs", "eventual consistency", "consensus", "replication", "partitioning"]
---

# Designing Data-Intensive Architecture (Martin Kleppmann)

This skill equips the agent with the architectural disciplines of Martin Kleppmann's *Designing Data-Intensive Applications (DDIA)* for architecting robust, scalable, fault-tolerant data storage and processing pipelines.

## 1. The Core Trinity: Reliability, Scalability, Maintainability
1. **Reliability (Fault Tolerance)**:
   - Make a clear distinction between a *fault* (one component deviating from spec) and a *failure* (the entire system ceasing to deliver service).
   - Design systems to absorb faults without causing total system failures.
2. **Scalability**:
   - Describe load mathematically: throughput (QPS), read/write ratios, fan-out, and percentile latencies (p95, p99, p99.9).
   - Design for bottlenecks: avoid monolithic table locks and high-contention sequential counters.
3. **Maintainability**:
   - Operability (telemetry, structured logs, observability).
   - Simplicity (managing and removing accidental complexity).
   - Evolvability (backward/forward schema compatibility with Protobuf/JSON schema evolution).

## 2. Transactions, Concurrency & Isolation Levels
1. **ACID in Practice**:
   - Understand the limits of database transaction promises:
   - **Read Committed**: Prevents dirty reads and dirty writes.
   - **Snapshot Isolation / Repeatable Read**: Prevents non-repeatable reads using MVCC (Multi-Version Concurrency Control).
   - **Serializable**: The gold standard preventing phantom reads, write skew, and lost updates.
2. **Preventing Write Skew**:
   - When transactions concurrently query a condition and update based on that condition (e.g. reserving a slot or court branch quota), use explicit locking (`SELECT FOR UPDATE`) or serializable isolation.

## 3. Idempotency & Distributed Logging
1. **Idempotency Keys**:
   - Every mutating request (payments, dockets, agent tool executions) must accept and record a unique `idempotency_key`.
   - On duplicate submission, return the previously computed result without re-executing business logic.
2. **The Log as the Source of Truth**:
   - Use an append-only Write-Ahead Log (WAL) or event stream as the primary source of truth.
   - Read models, search indices, and caches are derived materialized views asynchronously updated from the log.
3. **CQRS (Command Query Responsibility Segregation)**:
   - Separate write models (optimized for domain validation and ACID transactions) from read models (optimized for fast, denormalized querying).
