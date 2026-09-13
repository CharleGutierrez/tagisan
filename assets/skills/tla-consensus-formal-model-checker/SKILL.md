---
name: tla-consensus-formal-model-checker
description: "TLA+ & TLC formal model checking for distributed consensus protocols (Raft, Paxos, 2PC, PBFT), state-machine safety and liveness invariants."
version: 0.2.0
tags:
  - tla-plus
  - tlc
  - consensus
  - formal-methods
  - raft
  - paxos
  - two-phase-commit
  - pbft
  - model-checking
  - distributed-systems
compatibility: ">=0.2.0"
triggers:
  - "tla+"
  - "tlc"
  - "formal model checker"
  - "consensus"
  - "raft"
  - "paxos"
  - "2pc"
  - "pbft"
  - "safety invariant"
  - "liveness invariant"
  - "state space"
---

# TLA+ Consensus Formal Model Checker Skill

The `tla-consensus-formal-model-checker` skill provides rigorous specification and verification of distributed consensus protocols (Raft, Paxos, Two-Phase Commit, PBFT) using TLA+ and the TLC model checker, proving state-machine safety invariants and liveness properties under arbitrary message loss, delay, and reordering.

## Core Capabilities

1. **Consensus Specification Synthesis**: Synthesizes mathematically rigorous TLA+ specifications comprising state variables, typed initialization (`Init`), and non-deterministic action transitions (`Next`).
2. **Safety & Liveness Invariant Definition**: Codifies core distributed safety invariants (`ElectionSafety`, `LogMatching`, `Agreement`, `Consistency`) and temporal liveness properties.
3. **TLC Model Configuration Generation**: Synthesizes TLC configuration files (`.cfg`) declaring finite model constants, state constraints, and symmetry permutations.
4. **State Space Explosion Envelope Analysis**: Estimates reachable state space bounds, symmetry reduction factors ($N!$), BFS tree diameter, and TLC memory allocations.

## Strict Operational Invariants

- **ALWAYS**:
  - Express distributed state transitions as inductive predicates relating current state variables to unprimed and primed variables ($v$ and $v'$).
  - Specify explicit TypeOK invariants verifying that all variables remain within their mathematical domains across all reachable states.
  - Apply symmetry sets in TLC configurations (`SYMMETRY Permutations(Server)`) to collapse isomorphic state spaces by N factorial.
  - Verify both safety invariants (bad things never happen) and liveness properties (good things eventually happen under weak/strong fairness).

- **NEVER**:
  - NEVER omit stuttering steps in action specifications; ensure Next action allows $v' = v$ when composing sub-actions.
  - NEVER model unbounded message queues or unbounded terms without finite bounds in model checking configurations.
  - NEVER declare safety invariants without first validating model checking passes against known trivial counterexamples or mutants.
  - NEVER assume network delivery guarantees (FIFO or reliability) unless explicitly modeled as an invariant constraint.

- **MANDATORY**:
  - MANDATORY define atomic commit criteria (e.g. majority quorum floor(N/2) + 1 or PBFT 2f+1) within invariant definitions.
  - MANDATORY configure TLC runtime worker threads and memory allocations (`-workers`, `-maxSetSize`) scaled to the estimated state space diameter.
  - MANDATORY document the complete state transition DAG for all distributed action predicates (`RequestVote`, `AppendEntries`, `Prepare`, `Commit`).

- **STRICT_REJECT**:
  - STRICT_REJECT specifications with unconstrained state transitions that produce infinite state generation under TLC without progress.
  - STRICT_REJECT consensus protocol specifications that allow dual leaders in the same epoch/term or split-brain commits.
  - STRICT_REJECT TLC configs that omit vital safety invariants while evaluating temporal liveness properties.
