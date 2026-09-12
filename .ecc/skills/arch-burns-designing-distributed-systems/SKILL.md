---
name: arch-burns-designing-distributed-systems
description: "Patterns for containerized distributed systems: Sidecar, Ambassador, Adapter, Replicated Load-Balanced Services, Sharded Services, and Scatter/Gather."
triggers: ["brendan-burns", "designing-distributed-systems", "scatter-gather", "ambassador", "adapter", "sharded-services", "distributed-primitives"]
---

# arch-burns-designing-distributed-systems
> Based on **Designing Distributed Systems - Brendan Burns**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Parallel distributed search/aggregation queries must implement Scatter/Gather with strict timeout deadlines and partial result aggregation.**
2. **ALWAYS: Partition sharded services using consistent hashing to minimize data movement during node additions/removals.**
3. **NEVER: Allow a single slow leaf node in a Scatter/Gather cluster to hold up the aggregated client response past the deadline.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Scatter/Gather with root coordinator fan-out, leaf node execution, and deadline cancellation. Use consistent hashing for distributed sharded state.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Waiting indefinitely for the slowest leaf node in a scatter/gather query.**
- **Using naive modulo hashing (hash(key) % N) causing total rehash on node failure.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-burns-designing-distributed-systems"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
