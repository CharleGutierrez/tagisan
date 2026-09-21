---
name: harness-fuzz-benchmark-pro-max
description: Generative Fuzzing & Microbenchmarking Harness for Tagisan. Synthesizes property-based testing harnesses (proptest in Rust, hypothesis in Python), differential testing between alternative implementations, coverage-guided mutation fuzzing, and latency/memory microbenchmarks (p50, p99, memory allocation tracing). Enforces deterministic seed reproducibility, shrinking of minimal failing inputs, and zero false-negative exit code handling.
version: 1.0.0
tags:
  - property-testing
  - fuzzing
  - microbenchmarking
  - proptest
  - differential-testing
  - mutation-fuzzing
  - latency-profiling
  - memory-tracing
  - criterion
triggers:
  - fuzz-benchmark
  - proptest-harness
  - differential-testing
  - mutation-fuzzing
  - microbenchmark-harness
  - p99-latency-benchmark
  - memory-allocation-trace
  - property-based-testing
  - failure-shrinking
compatibility: ">=0.2.0"
---

# Generative Fuzzing & Microbenchmarking Harness: Autonomous Invariant Verification & Performance Profiling

## Purpose & Scope
Passing conventional unit tests is insufficient to ensure system reliability under adversarial conditions and high throughput. Rare edge cases, integer overflows, memory leaks, and tail-latency regressions remain hidden until exposed in production.

The `harness-fuzz-benchmark-pro-max` skill autonomously synthesizes property-based test suites, differential testing harnesses comparing dual implementations, coverage-guided mutation fuzzers, and micro-benchmarks measuring p50/p99 latency and heap allocation volumes.

---

## 1. Operational Invariants

```
+---------------------------------------------------------------------------------------------------+
|                              FUZZING & BENCHMARKING INVARIANTS                                    |
+---------------------------------------------------------------------------------------------------+
|  1. DETERMINISTIC SEED REPRODUCIBILITY                                                            |
|     Every fuzzing and property testing execution MUST log its initial random seed. Providing the  |
|     same seed must reproduce the exact sequence of generated inputs and failure points.           |
+---------------------------------------------------------------------------------------------------+
|  2. MINIMAL FAILING INPUT SHRINKING                                                               |
|     When a property violation or crash is discovered, the harness MUST automatically shrink the   |
|     input to the minimal reproducible counterexample before generating the failure diagnostic.    |
+---------------------------------------------------------------------------------------------------+
|  3. ZERO FALSE-NEGATIVE EXIT CODE HANDLING                                                        |
|     Any panic, assertion failure, memory violation (ASan/MSan), or exit code mismatch MUST be     |
|     surfaced as a hard non-zero exit code. Silently swallowed errors are strictly forbidden.       |
+---------------------------------------------------------------------------------------------------+
|  4. ZERO ALLOCATION VALIDATION FOR HOT PATHS                                                      |
|     Critical fast paths (e.g. zero-copy parsers, crypto inner loops) must be verified with custom |
|     allocator hooks to enforce zero heap allocations per operation.                               |
+---------------------------------------------------------------------------------------------------+
|  5. STATISTICAL OUTLIER FILTERING & RIGOR                                                         |
|     Microbenchmarks must discard warmup iterations and compute robust statistical distributions   |
|     (min, p50, p90, p99, max, standard deviation) rather than relying on simple averages.         |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Generative Fuzzing & Property-Based Testing

### Rust `proptest` Property Harness
```rust
//! Production Property-Based Testing Harness synthesized by Tagisan
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 10000,
        rng_seed: proptest::test_runner::RngSeed::Fixed([42; 32]),
        .. ProptestConfig::default()
    })]

    #[test]
    fn test_roundtrip_codec_invariance(
        payload in prop::collection::vec(any::<u8>(), 0..65536),
        flags in any::<u16>()
    ) {
        let encoded = encode_packet(&payload, flags);
        let decoded = decode_packet(&encoded).expect("Valid payload must decode");

        // Mathematical invariants
        prop_assert_eq!(decoded.payload, payload, "Payload must be preserved exactly");
        prop_assert_eq!(decoded.flags, flags, "Flags must match");
    }

    #[test]
    fn test_parser_never_panics_on_arbitrary_bytes(
        fuzz_bytes in prop::collection::vec(any::<u8>(), 0..2048)
    ) {
        // Must return Ok or Err, NEVER panic
        let _ = parse_untrusted_input(&fuzz_bytes);
    }
}
```

---

## 3. Differential Testing Architecture

Differential testing executes two independent implementations against identical generated inputs to detect divergence in logic, precision, or error handling.

```
+---------------------------------------------------------------+
|                    DIFFERENTIAL TEST RUNNER                   |
+---------------------------------------------------------------+
                               |
                +--------------+--------------+
                | Random / Mutated Input Gen  |
                +--------------+--------------+
                               |
        +----------------------+----------------------+
        |                                             |
        v                                             v
+-----------------------+                   +-----------------------+
|  Reference Oracle     |                   |  Candidate Harness    |
|  (e.g., Python / C)   |                   |  (e.g., Rust / Bun)   |
+-----------------------+                   +-----------------------+
        |                                             |
        v                                             v
  Output Result A                               Output Result B
        |                                             |
        +----------------------+----------------------+
                               |
                               v
               +-------------------------------+
               | Invariant Comparator          |
               | - Exit code match?            |
               | - JSON data equality?         |
               | - Precision within 1e-9?      |
               +-------------------------------+
```

---

## 4. Latency & Memory Microbenchmarking Engine

```rust
//! High-Precision Nanosecond Microbenchmark Harness
use std::time::Instant;

pub struct BenchmarkReport {
    pub iterations: usize,
    pub p50_ns: u64,
    pub p90_ns: u64,
    pub p99_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
    pub total_allocations: usize,
}

pub fn run_benchmark<F: FnMut()>(mut operation: F, warmup_iters: usize, measure_iters: usize) -> BenchmarkReport {
    // 1. Warmup phase
    for _ in 0..warmup_iters {
        operation();
    }

    // 2. Measurement phase
    let mut timings = Vec::with_capacity(measure_iters);
    for _ in 0..measure_iters {
        let start = Instant::now();
        operation();
        let elapsed = start.elapsed().as_nanos() as u64;
        timings.push(elapsed);
    }

    timings.sort_unstable();

    let p50 = timings[timings.len() * 50 / 100];
    let p90 = timings[timings.len() * 90 / 100];
    let p99 = timings[timings.len() * 99 / 100];
    let min = timings[0];
    let max = timings[timings.len() - 1];

    BenchmarkReport {
        iterations: measure_iters,
        p50_ns: p50,
        p90_ns: p90,
        p99_ns: p99,
        min_ns: min,
        max_ns: max,
        total_allocations: 0, // Hooked to tracking allocator
    }
}
```

---

## 5. Autonomous Fuzz & Benchmark Workflow

1. **Target Inspection**: Ingest function signatures and identify critical stateful or parsing operations.
2. **Strategy Generation**: Synthesize `proptest` or `hypothesis` generators with domain-specific constraints (e.g. valid UTF-8, valid IP addresses, bounded lengths).
3. **Fuzz Execution**: Execute 10,000+ iterations with deterministic seed recording.
4. **Counterexample Shrinking**: On failure, shrink input to minimal reproducing test case and save fixture.
5. **Microbenchmark Profiling**: Run statistical benchmark, generate p50/p99 latency distribution, and verify allocation bounds.
