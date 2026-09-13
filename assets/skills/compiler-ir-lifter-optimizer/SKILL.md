---
name: compiler-ir-lifter-optimizer
description: LLVM-IR, Cranelift, and assembly-level optimization. Audits pointer aliasing penalties, eliminates branches, guides hot-loop register allocation to prevent spills, and enforces verified inline assembly safeguards.
version: 1.0.0
tags:
  - compiler-optimization
  - llvm-ir
  - cranelift
  - assembly
  - pointer-aliasing
  - branch-elimination
  - register-allocation
  - inline-assembly
triggers:
  - compiler-optimizer
  - llvm-ir
  - cranelift
  - assembly-optimization
  - pointer-aliasing
  - branch-elimination
  - register-spill
  - inline-asm
compatibility: ">=0.2.0"
---

# Compiler IR Lifter & Assembly Optimizer: Microarchitectural Hot-Loop Synthesis

## Purpose & Scope
Modern optimizing compilers (LLVM, Cranelift, GCC) emit sub-optimal machine code when hindered by ambiguous pointer aliasing, unpredictable data-dependent branching, memory reloads across loop iterations, or register pressure leading to stack spills.

This skill equips autonomous systems with mechanical-sympathy compiler analysis. It inspects emitted intermediate representations (LLVM-IR, assembly), guarantees pointer disjointness via `restrict` / non-overlapping slice borrows, synthesizes branchless arithmetic predicates, eliminates memory reload penalties, and configures sound inline assembly safeguards.

---

## 1. Operational Invariants (Compiler Optimization Protocol)

### Invariant 1: Pointer Aliasing Elimination & Restrict Disjointness
- **MANDATORY**: All pointer arguments and slice buffers operating within tight inner loops must be provably non-overlapping. Use Rust's exclusive mutable borrow semantics (`&mut [T]`) or slice splitting (`split_at_mut`) to enable LLVM to emit `noalias` metadata.
- **ALWAYS**: Avoid multiple raw pointers (`*mut T`, `*const T`) in loops without explicit `restrict` semantics or compiler-fence barriers.
- **STRICT_REJECT**: Reject any loop structure where the compiler is forced to issue redundant memory loads (`LOAD` following `STORE`) due to unproven aliasing hazards.

### Invariant 2: Branch Elimination & Predicated Execution
- **ALWAYS**: Convert data-dependent inner loop branches (`if`/`else`) into branchless conditional moves (`cmov`), bitwise selection masks, or arithmetic multiplexing.
- **NEVER**: Allow cold error-handling paths or debug assertions to pollute hot loop instruction caches. Mark error branches with `#[cold]` and cold functions with `#[inline(never)]`.
- **MANDATORY**: Hot path functions must be annotated with `#[inline(always)]` to eliminate call/return ABI overhead and facilitate inter-procedural vectorization passes.

### Invariant 3: Hot-Loop Register Allocation & Spill Elimination
- **MANDATORY**: Hot loop working sets must fit within target architecture general-purpose and vector register files (e.g. 16 registers on x86_64, 32 registers on AArch64).
- **STRICT_REJECT**: Reject loop bodies that cause stack spilling (excessive `MOV` instructions to/from `[rsp]`) inside inner iteration bodies. Restructure variables into packed structs or unroll loops with bounded depth.
- **ALWAYS**: Unroll loops to match processor load/store pipeline depth (typically 2 to 4 unrolled iterations) while preserving instruction cache locality.

### Invariant 4: Inline Assembly & Target Architecture Safeguards
- **MANDATORY**: Every inline assembly block (`core::arch::asm!`) must declare exact input registers (`in("rax")`), output registers (`out("rdx")`), and clobber lists (`clobber_abi("C")` or `lateout`).
- **NEVER**: Execute inline assembly with implicit register side-effects or unannounced memory mutations. Memory mutations must be declared via `inout("memory")` or compiler fences.
- **ALWAYS**: Guard architecture-specific intrinsics and assembly behind target configuration gates (`#[cfg(target_arch = "...")]`) with safe, portable software fallbacks.

---

## 2. Canonical Optimization Patterns

### Branchless Clamp & Disjoint Slice Transform (Rust)
```rust
#[inline(always)]
pub fn transform_disjoint_buffers(src: &[f32], dst: &mut [f32], threshold: f32) {
    assert_eq!(src.len(), dst.len());
    let len = src.len();
    
    // Non-overlapping slices guarantee LLVM emits noalias
    for i in 0..len {
        let val = src[i];
        // Branchless conditional selection compiles to single cmov / vblend
        let mask = (val > threshold) as u32;
        let clamped = if mask != 0 { threshold } else { val };
        dst[i] = clamped;
    }
}

#[cold]
#[inline(never)]
pub fn handle_out_of_bounds_trap(code: u32) -> ! {
    panic!("Fatal out-of-bounds trap: error code 0x{:08x}", code);
}
```

---

## 3. Tool Invocations

Use `compiler_ir_optimizer` to inspect compiler bottlenecks and generate optimized machine code:
- `compiler_ir_optimizer(action: "analyze_assembly", code: "pub fn dot(a: &[f32], b: &[f32]) ...", target_arch: "x86_64")`
- `compiler_ir_optimizer(action: "detect_aliasing_penalties", code: "...", target_arch: "aarch64")`
- `compiler_ir_optimizer(action: "generate_optimizations", code: "...", target_arch: "x86_64")`
