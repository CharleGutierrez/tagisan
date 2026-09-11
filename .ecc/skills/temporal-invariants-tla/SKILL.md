---
name: temporal-invariants-tla
description: "Temporal Invariants and Formal Specification (Leslie Lamport and Hillel Wayne): safety invariants, liveness guarantees, state space exhaustion, and race-free concurrent and distributed state modeling."
triggers: ["tla", "lamport", "hillel wayne", "temporal invariants", "formal specification", "safety invariant", "liveness invariant", "state space", "concurrency model", "deadlock free"]
---

# Temporal Invariants and State Space Verification (Lamport / Wayne)

This skill equips the agent with formal specification disciplines from Leslie Lamport's TLA+ and Hillel Wayne's practical state modeling to verify concurrent, async, and distributed workflows against deadlocks and race conditions.

## 1. Safety vs. Liveness Invariants
1. **Safety Properties ('Nothing bad happens')**:
   - Invariants that must hold true in *every reachable state* of the system.
   - Examples:
     - Account balance never drops below zero.
     - Two agents never hold write locks on the same task simultaneously.
     - No dangling task states in a DAG scheduler.
2. **Liveness Properties ('Something good eventually happens')**:
   - Invariants guaranteeing that the system makes forward progress and does not get stuck in infinite loops, livelocks, or unresolvable wait states.
   - Examples:
     - Every submitted task is eventually either executed or marked failed.
     - A released lock is eventually granted to a waiting thread.

## 2. State Space Modeling Before Coding
1. **State Variables**: Identify the minimal, discrete tuple of variables that completely define the system state:
   - State = <Status, ActiveTasks, LockOwner, ResourceTokens>.
2. **Actions as State Transitions**:
   - Model all mutations as pure transition functions: State_next = Action(State_current).
   - Explicitly define the Enabled Condition (guard) under which each action is valid.
3. **Exhaustive State-Space Thinking**:
   - Consider all possible interleavings of concurrent actions: what happens if Action A runs halfway, then Action B intervenes?
   - Eliminate non-deterministic interleavings using transactional boundaries or atomic compare-and-swap semantics.

## 3. Translating Formal Invariants to Implementation
1. **Type-Level Invariants**:
   - Make illegal states unrepresentable in the type system (e.g. using Rust enums or discriminated unions instead of loose boolean flags).
2. **Runtime Assertions and Invariant Guards**:
   - Embed debug_assert! checks asserting safety invariants at entry and exit of state machine transitions.
3. **Model-Based Property Tests**:
   - Generate random sequences of actions and verify that the safety invariant holds across thousands of pseudo-random event sequences.
