---
name: rust-proptest-fuzzing
description: Property-based invariant testing, fuzzing with proptest/quickcheck, and minimal failure shrink analysis
---

# Rust Property-Based Testing & Invariant Fuzzing

## Fuzzing Protocols
1. **Invariant Identification**: Define mathematical or structural invariants (e.g., `decode(encode(x)) == x`, `sort(x).is_sorted()`, `a + b == b + a`).
2. **Generator Strategies**: Author custom `proptest::strategy::Strategy` implementations that span entire domains including edge cases (empty collections, MAX/MIN bounds, NaN, UTF-8 surrogate code points).
3. **Minimal Failing Vector Analysis**: Utilize proptest's automated shrinking algorithm to isolate the exact minimal reproduction input on failure.
