---
name: arch-shapira-kafka-definitive
description: "Distributed commit log architecture: Partitioning mechanics, consumer group rebalancing, exact partition causal ordering, producer ACKs (all vs 1), and compacted topics."
triggers: ["shapira-kafka", "kafka-architecture", "commit-log", "consumer-groups", "partition-key", "producer-acks", "compacted-topics", "rebalance-protocol"]
---

# arch-shapira-kafka-definitive
> Based on **Kafka: The Definitive Guide - Gwen Shapira, Todd Palino, Rajeev Sivaram, Krit Petty**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Partition keys must map directly to entity aggregate root IDs to guarantee strictly sequential causal order per entity.**
2. **ALWAYS: Set producer `acks=all` (min.insync.replicas >= 2) for mission-critical events to guarantee durability against broker failures.**
3. **NEVER: Commit consumer offsets before processing the batch to completion unless at-most-once loss is explicitly permitted.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design event topologies with dedicated topic partitions, consumer group rebalancing strategies, and key-based ordering. Configure compaction for state-store topics.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using random partition keys when domain entities require sequential event processing.**
- **Committing consumer offsets asynchronously before executing side effects.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-shapira-kafka-definitive"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
