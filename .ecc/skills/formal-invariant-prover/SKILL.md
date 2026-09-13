---
name: formal-invariant-prover
description: Property-based testing, fuzzing, and formal verification engine utilizing proptest, quickcheck, and Kani bounded model checking. Transcends happy-path unit tests to mathematically prove state transitions, boundary limits, and inductive invariants.
version: 1.0.0
tags:
  - formal-verification
  - proptest
  - quickcheck
  - kani
  - property-based-testing
  - bounded-model-checking
  - inductive-invariants
triggers:
  - formal-verification
  - proptest
  - quickcheck
  - kani
  - invariant-prover
  - property-based-testing
  - bounded-model-checking
  - inductive-invariants
  - panic-freedom
compatibility: ">=0.2.0"
---

# Formal Invariant Prover: Property-Based Testing & Model Checking

## Purpose & Scope
Traditional unit tests only verify hand-picked examples on expected happy paths, routinely missing boundary edge-cases, integer overflows, off-by-one errors, and concurrency anomalies.

The Formal Invariant Prover elevates engineering quality from empirical sampling to rigorous mathematical proof. Using `proptest`, `quickcheck`, and Kani Bounded Model Checking (BMC), this skill mandates the construction of generative strategies, algebraic properties, inductive invariants, and bounded proofs of panic-freedom.

---

## 1. Operational Invariants (Formal Verification Protocol)

### Invariant 1: Beyond Happy-Path Unit Testing
- **STRICT_REJECT**: Never consider a critical parser, cryptographic routine, financial calculator, state machine, or serialization engine verified using only static assertion examples (e.g. `assert_eq!(add(2, 2), 4)`).
- **MANDATORY**: Critical algorithms must be accompanied by property-based tests asserting algebraic invariants over arbitrarily generated inputs across at least 1,000 iterations.

### Invariant 2: Exhaustive Boundary Singularities
- Every generator strategy must explicitly include extreme edge cases and boundary limits:
  - Integers: `0`, `1`, `-1`, `T::MIN`, `T::MAX`, `T::MIN + 1`, `T::MAX - 1`.
  - Floating Point: `0.0`, `-0.0`, `f64::MIN_POSITIVE`, `f64::EPSILON`, `f64::INFINITY`, `f64::NEG_INFINITY`, `f64::NAN`, subnormal values.
  - Collections: Empty (`[]`, `""`), singletons, powers of two sizes, large buffers (>64KB).
  - Strings: Valid UTF-8, null bytes, bidirectional control characters, emoji sequences, unnormalized unicode (NFC/NFD).

### Invariant 3: Inductive Invariant & Property Axioms
- Every formal verification suite must assert one or more of the fundamental algebraic properties:
  1. **Roundtrip Invertibility**: `decode(encode(x)) == Ok(x)`.
  2. **Idempotence**: `f(f(x)) == f(x)`.
  3. **Commutativity / Associativity**: `f(a, b) == f(b, a)` and `f(f(a, b), c) == f(a, f(b, c))`.
  4. **Monotonicity**: `a <= b ==> f(a) <= f(b)`.
  5. **Conservation of Invariants**: Pre-conditions and post-conditions preserve balance / conservation laws (e.g., total sum across accounts before transfer equals total sum after transfer).

### Invariant 4: Bounded Model Checking & Panic-Freedom (Kani)
- For safety-critical, `unsafe`, or concurrent code blocks, author Kani verification harnesses (`#[kani::proof]`):
  - Mark symbolic nondeterministic inputs with `kani::any()`.
  - Bound unbounded loops with `#[kani::unwind(N)]`.
  - Mathematically prove absence of arithmetic overflow, out-of-bounds pointer indexing, and panics across all possible execution paths.

---

## 2. Canonical Proptest & Kani Patterns

### Proptest Roundtrip Property
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_codec_roundtrip(val in any::<Vec<u8>>()) {
        let encoded = encode(&val);
        let decoded = decode(&encoded).expect("Decoding must never fail on valid encoded bytes");
        prop_assert_eq!(decoded, val);
    }

    #[test]
    fn test_sorting_preserves_length_and_order(mut list in prop::collection::vec(any::<i64>(), 0..1000)) {
        let original_len = list.len();
        list.sort();
        prop_assert_eq!(list.len(), original_len);
        prop_assert!(list.windows(2).all(|w| w[0] <= w[1]));
    }
}
```

### Kani Panic-Freedom Harness
```rust
#[cfg(kani)]
#[kani::proof]
#[kani::unwind(16)]
fn verify_buffer_safe_slice() {
    let size: usize = kani::any();
    kani::assume(size <= 1024);
    let buffer = vec![0u8; size];
    
    let offset: usize = kani::any();
    let len: usize = kani::any();
    
    if offset.saturating_add(len) <= buffer.len() {
        let slice = &buffer[offset..offset + len];
        assert_eq!(slice.len(), len);
    }
}
```
