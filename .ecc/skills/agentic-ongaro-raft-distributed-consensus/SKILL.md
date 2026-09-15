---
name: agentic-ongaro-raft-distributed-consensus
description: "Leader election, log replication, safety invariants, randomized election timeouts, joint consensus reconfiguration, and state machine replication."
triggers: ["ongaro", "ousterhout", "raft-consensus", "leader-election", "log-replication", "state-machine-replication", "randomized-timeouts"]
---

# agentic-ongaro-raft-distributed-consensus
> Based on **In Search of an Understandable Consensus Algorithm (Raft) - Diego Ongaro & John Ousterhout**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Raft State Invariants: Election Safety (at most one leader per term); Leader Append-Only; Log Matching; Leader Completeness; State Machine Safety.**
2. **Quorum Majority Rule: A leader can commit a log entry only after it is replicated on a strict majority of nodes: floor(N/2) + 1.**
3. **Randomized Election Timeouts: Split-vote prevention by randomizing election timeouts (e.g. 150ms-300ms).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement leader election and state machine replication for multi-agent clusters using Raft consensus. Guarantee quorum agreement before committing configuration changes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Split-brain scenarios caused by committing log entries without majority quorum confirmation.**
- **Fixed election timeouts causing perpetual split-vote election ties.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ongaro"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
