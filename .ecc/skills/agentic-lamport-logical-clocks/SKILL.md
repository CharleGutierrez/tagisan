---
name: agentic-lamport-logical-clocks
description: "Partial orderings, happens-before relation (->), logical timestamps, vector clocks, total ordering consistency, and distributed state machines."
triggers: ["lamport", "logical-clocks", "happens-before", "vector-clocks", "lamport-timestamps", "distributed-ordering"]
---

# agentic-lamport-logical-clocks
> Based on **Time, Clocks, and the Ordering of Events in a Distributed System - Leslie Lamport**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Happens-Before Relation (->): If a and b are in the same process and a occurs before b, then a -> b. If a is send and b is receive, a -> b.**
2. **Lamport Timestamp Update: C(e) = max(C_local, C_msg) + 1, establishing a strict partial order across distributed events.**
3. **Vector Clocks: V_i[j] tracks agent i's knowledge of agent j's logical time, enabling detection of causal vs concurrent events.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Order agent swarm actions and messages using Lamport timestamps or vector clocks to guarantee causal consistency without relying on unsynchronized wall clocks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using system physical wall clocks (SystemTime) to order distributed events, causing clock drift corruption.**
- **Assuming concurrent events have a natural causal order without vector clock verification.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "lamport"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
