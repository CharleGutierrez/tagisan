---
name: simd-compute-auto-vectorizer
description: Portable SIMD vectorization & tensor loop speedup. Analyzes loop dependencies, eliminates vectorization inhibitors (branching, non-contiguous memory), rewrites scalar loops into portable SIMD / chunked lane arrays, and benchmarks with Criterion.
version: 1.0.0
tags:
  - simd
  - vectorization
  - avx2
  - avx512
  - neon
  - tensor-loops
  - mechanical-sympathy
  - criterion-benchmarking
triggers:
  - simd
  - vectorization
  - auto-vectorizer
  - loop-vectorization
  - avx2
  - avx512
  - neon
  - tensor-compute
  - lane-width
  - simd-speedup
compatibility: ">=0.2.0"
---

# SIMD Compute & Auto-Vectorizer: Portable High-Throughput Tensor Acceleration

## Purpose & Scope
Modern compute-intensive workloads—including matrix multiplication, tensor convolutions, audio/signal DSP, cryptographic hashing, and vector search—suffer catastrophic 4x to 64x throughput degradations when executed via naive scalar loops. Compilers often fail to auto-vectorize loops due to hidden pointer aliasing, non-contiguous strides, conditional branching, or complex induction variables.

This skill equips autonomous agents with systematic, mechanical-sympathy SIMD vectorization. By analyzing loop-carried data dependencies, stripping inhibitors, reorganizing memory for aligned streaming, and generating explicit portable chunked vector lanes (`std::simd` / AVX2 / AVX-512 / NEON), agents unlock maximum CPU throughput with verified numerical equivalence.

---

## 1. Operational Invariants (SIMD Vectorization Protocol)

### Invariant 1: Loop Dependency Analysis & Independence Proof
- **MANDATORY**: Prior to vectorization, analyze loop-carried data dependencies (Read-After-Write, Write-After-Read, Write-After-Write). Vectorization across lanes requires that loop iteration $i$ is mathematically independent of iteration $i-k$.
- **STRICT_REJECT**: Reject naive auto-vectorization when a true loop-carried dependency exists, unless transformed into an associative algebraic reduction (e.g., parallel horizontal vector summation or prefix scan).

### Invariant 2: Elimination of Vectorization Inhibitors
- **ALWAYS**: Eliminate branching control flow (`if`/`else`), early breaks, non-contiguous indexing (`arr[idx[i]]`), and polymorphic function calls within inner loop bodies.
- **NEVER**: Allow conditional branches inside hot SIMD loops. Branching must be converted into branchless bitwise masks, blending operations (`select` / `blendv`), or arithmetic predicates.
- **ALWAYS**: Ensure pointer aliasing hazards are disproven using `restrict`, Rust slices (`&[T]`, `&mut [T]` with non-overlapping borrow guarantees), or explicit chunk iterators (`chunks_exact`).

### Invariant 3: Portable SIMD & Chunked Lane Architecture
- **MANDATORY**: Structure vector transformations into dual phases:
  1. **Main Vector Body**: Iterate over data in chunks of $W$ elements matching hardware lane width ($W=4$ for 64-bit, $W=8$ for 32-bit on AVX2, $W=16$ on AVX-512, $W=64$ for 8-bit).
  2. **Scalar Remainder Tail**: Process the remaining $N \pmod W$ elements using an unrolled scalar loop to guarantee exact boundary handling without buffer overflows.
- **ALWAYS**: Prefer portable SIMD abstractions (`std::simd`, `packed_simd`, or portable lane arrays) with fallback to native intrinsics (`core::arch::x86_64` or `core::arch::aarch64`) when target-specific microarchitecture optimizations (e.g. FMA3, dot product) are required.

### Invariant 4: Empirical Benchmark Verification with Criterion
- **MANDATORY**: Every vectorization rewrite must be accompanied by a Criterion benchmark comparing the scalar baseline against the SIMD implementation across three input size regimes: Small ($N=64$), Medium ($N=1,024$), and Large ($N=65,536$).
- **STRICT_REJECT**: Reject any SIMD transformation that introduces negative speedup or regressions caused by register spill, excessive pack/unpack overhead, or cache thrashing.

---

## 2. Canonical SIMD Transformation Patterns

### Portable Chunked Lane Vectorization (Rust)
```rust
#[inline(always)]
pub fn vector_add_f32(a: &[f32], b: &[f32], out: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());

    const LANES: usize = 8; // AVX2 256-bit / 32-bit = 8 lanes
    let (a_chunks, a_tail) = a.as_chunks::<LANES>();
    let (b_chunks, b_tail) = b.as_chunks::<LANES>();
    let (out_chunks, out_tail) = out.as_chunks_mut::<LANES>();

    for ((a_vec, b_vec), out_vec) in a_chunks.iter().zip(b_chunks.iter()).zip(out_chunks.iter_mut()) {
        for i in 0..LANES {
            out_vec[i] = a_vec[i] + b_vec[i];
        }
    }

    // Scalar tail handling
    for ((a_val, b_val), out_val) in a_tail.iter().zip(b_tail.iter()).zip(out_tail.iter_mut()) {
        *out_val = *a_val + *b_val;
    }
}
```

### AVX2 Explicit FMA Dot Product
```rust
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[inline]
pub unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len();
    let mut sum_vec = _mm256_setzero_ps();
    let chunks = len / 8;

    for i in 0..chunks {
        let va = _mm256_loadu_ps(a.as_ptr().add(i * 8));
        let vb = _mm256_loadu_ps(b.as_ptr().add(i * 8));
        sum_vec = _mm256_fmadd_ps(va, vb, sum_vec);
    }

    // Horizontal reduction
    let mut buffer = [0.0f32; 8];
    _mm256_storeu_ps(buffer.as_mut_ptr(), sum_vec);
    let mut total: f32 = buffer.iter().sum();

    // Remainder tail
    for i in (chunks * 8)..len {
        total += a[i] * b[i];
    }
    total
}
```

---

## 3. Tool Invocations

Use `simd_vectorizer` to analyze, vectorize, and estimate benchmark speedups:
- `simd_vectorizer(action: "analyze", code: "...", data_type: "f32", target_isa: "avx2")`
- `simd_vectorizer(action: "vectorize", code: "...", data_type: "f32", target_isa: "avx2")`
- `simd_vectorizer(action: "benchmark_estimate", code: "...", data_type: "f32", target_isa: "avx2")`
