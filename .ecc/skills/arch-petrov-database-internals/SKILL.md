---
name: arch-petrov-database-internals
description: "Storage engines and distributed consensus: B-Trees, Log-Structured Merge (LSM) Trees, Memtables, SSTables, Write-Ahead Logging (WAL), Paxos, and Raft consensus."
triggers: ["petrov", "database-internals", "lsm-tree", "sstable", "memtable", "wal", "raft-consensus", "paxos", "storage-engine"]
---

# arch-petrov-database-internals
> Based on **Database Internals: A Deep Dive into How Distributed Data Systems Work - Alex Petrov**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: High-throughput write pipelines must utilize append-only Write-Ahead Logging (WAL) and memory table flushing before persisting sorted string tables (SSTables).**
2. **ALWAYS: Consensus clusters (Raft/Paxos) require a strict majority quorum ((N/2) + 1) to commit log entries.**
3. **NEVER: Commit state changes to disk without fsync flushing the WAL to non-volatile storage.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Architect storage layers based on read/write asymmetry (B-Trees for point lookups, LSM-Trees for write bursts). Enforce Raft quorum mechanics for replicated leader election.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Modifying database files in-place without write-ahead logging.**
- **Allowing split-brain leader elections without majority quorum.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-petrov-database-internals"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
