---
name: kani-rust-formal-verifier
description: AWS Kani bounded model checking and CBMC/SMT formal verification for Rust. Proves mathematical panic freedom, verifies inductive loop invariants and pointer validity, audits unwraps and indexing, and synthesizes exhaustive proof harnesses with #[kani::proof] and kani::any().
version: 1.0.0
tags:
  - kani
  - formal-verification
  - smt-solver
  - cbmc
  - bounded-model-checking
  - zero-panic
  - invariant-proofs
  - rust-verification
triggers:
  - kani-rust-formal-verifier
  - kani-verifier
  - formal-verification
  - zero-panic-proof
  - bounded-model-checking
  - kani-proof-harness
compatibility: ">=0.2.0"
---

# AWS Kani Bounded Model Checking & Formal Verification Sentinel

## Purpose & Scope
High-assurance systems (cryptographic primitives, financial transaction ledgers, autonomous avionics, and low-level concurrency engines) cannot rely on fuzzing or randomized testing alone. Edge cases hiding in 64-bit state spaces escape sampling and trigger panics, arithmetic overflows, and undefined behaviors in mission-critical environments.

This skill synthesizes and validates mathematical proofs of program correctness using AWS Kani (built on the CBMC SMT solver backend). It transforms Rust functions into bit-precise first-order logic formulas, proving exhaustive panic freedom, verifying pointer validities and loop induction invariants, and generating deterministic verification harnesses.

---

## 1. Operational Invariants (Kani Formal Verification Protocol)

### Invariant 1: Mathematical Proof of Panic Freedom & Panic Path Elimination
- **MANDATORY**: Critical production paths must possess formal proofs demonstrating that no execution path can evaluate to a panic (`unwrap()`, `expect()`, out-of-bounds indexing `[i]`, division by zero `/ 0`, or unhandled integer overflow).
- **ALWAYS**: Audit source code for unverified panicking primitives, replacing them with checked variants (`get()`, `checked_div()`, `checked_add()`) or proving inductive constraints via `kani::assume()`.
- **STRICT_REJECT**: Reject any production code claiming formal verification if unconstrained panic paths remain reachable under valid non-deterministic harness inputs.

### Invariant 2: Exhaustive Bounded Induction & Loop Unrolling
- **MANDATORY**: Every loop verified with Kani must specify an explicit unwinding bound via `#[kani::unwind(N)]` that covers the maximal induction limit of the algorithm.
- **ALWAYS**: Verify inductive loop termination and loop invariants asserting that state variables remain within verified safe intervals during each iteration.
- **STRICT_REJECT**: Reject verification harnesses with artificial under-unwinding that truncate loop execution without proving an inductive termination lemma.

### Invariant 3: Pointer Validity & Spatial/Temporal Memory Safety Proofs
- **MANDATORY**: All unsafe transmutes, raw pointer slices (`std::slice::from_raw_parts`), and FFI allocations must be formally proved safe: pointers are non-null, properly aligned, point to initialized memory of sufficient length, and do not alias mutably.
- **NEVER**: Allow pointer arithmetic to exceed allocation boundaries under any non-deterministic offset.
- **STRICT_REJECT**: Reject any unsafe pointer routine that fails spatial bounds verification in Kani's pointer safety model.

### Invariant 4: Safe Proof Harness Generation & Non-Deterministic Inoculation
- **MANDATORY**: Proof harnesses must use `kani::any()` to generate completely non-deterministic values across the entire valid domain of inputs.
- **ALWAYS**: Constrain input domains using `kani::assume(condition)` strictly to model true domain preconditions, and verify output guarantees using `kani::assert!(invariant, "proof assertion")`.
- **STRICT_REJECT**: Reject test harnesses that hardcode concrete sample inputs instead of exploring the exhaustive symbolic state space.

---

## 2. Canonical Kani Verification Patterns

### Exhaustive Panic Freedom Harness with Bounded Induction
```rust
#[cfg(kani)]
mod verification {
    use super::*;

    /// Non-deterministic proof harness verifying zero panics in RingBuffer::push
    #[kani::proof]
    #[kani::unwind(16)]
    pub fn verify_ring_buffer_infallible_push() {
        let capacity: usize = kani::any();
        kani::assume(capacity > 0 && capacity <= 16 && capacity.is_power_of_two());

        let mut buffer = BoundedRingBuffer::<u64, 16>::new(capacity);
        let items_to_push: usize = kani::any();
        kani::assume(items_to_push <= 32);

        for _ in 0..items_to_push {
            let val: u64 = kani::any();
            let result = buffer.try_push(val);

            if buffer.len() == capacity {
                kani::assert!(result.is_err(), "Must reject when buffer is at capacity");
            } else {
                kani::assert!(result.is_ok(), "Must succeed when buffer has remaining capacity");
            }

            kani::assert!(buffer.len() <= capacity, "Length must never exceed capacity");
        }
    }

    /// Verifying pointer alignment and slice validity
    #[kani::proof]
    pub fn verify_safe_slice_transmutation() {
        let raw_bytes: [u8; 8] = kani::any();
        let ptr = raw_bytes.as_ptr();

        if (ptr as usize) % core::mem::align_of::<u64>() == 0 {
            let val = unsafe { *(ptr as *const u64) };
            kani::assert!(val == u64::from_ne_bytes(raw_bytes), "Transmuted value must match native bytes");
        }
    }
}
```


