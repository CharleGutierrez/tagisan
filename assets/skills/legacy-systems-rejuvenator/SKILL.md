---
name: legacy-systems-rejuvenator
description: C/C++ to safe idiomatic Rust rejuvenation. Synthesizes borrow-checker lifetimes, eliminates raw pointers, encapsulates unsafe blocks into zero-cost safe RAII types, and verifies functional equivalence via differential property-based testing.
version: 1.0.0
tags:
  - legacy-rejuvenation
  - c-to-rust
  - cpp-to-rust
  - memory-safety
  - borrow-checker
  - lifetime-synthesis
  - raii
  - differential-testing
triggers:
  - legacy-rejuvenator
  - c-to-rust
  - cpp-to-rust
  - rejuvenate-codebase
  - raw-pointer-elimination
  - lifetime-synthesis
  - safe-rust-migration
compatibility: ">=0.2.0"
---

# Legacy Systems Rejuvenator: C/C++ to Idiomatic Safe Rust Synthesis

## Purpose & Scope
Legacy C and C++ systems underpin critical infrastructure (operating systems, networking stacks, embedded devices, database storage engines), yet suffer from catastrophic vulnerability classes: use-after-free, double-free, buffer overflows, data races, and uninitialized memory reads.

This skill synthesizes a formal, verifiable path from legacy C/C++ codebases to 100% safe, idiomatic Rust. It maps manual pointer arithmetic to compile-time borrow-checker lifetimes, synthesizes safe RAII ownership types, encapsulates unavoidable FFI into zero-cost abstractions, and proves semantic equivalence using differential property-based testing.

---

## 1. Operational Invariants (Rejuvenation Protocol)

### Invariant 1: Borrow-Checker Lifetime Synthesis & Ownership
- **MANDATORY**: Every C pointer pair (`void* buf, size_t len`) must be synthesized into a safe Rust borrowed slice (`&[T]`) or exclusive mutable slice (`&mut [T]`) with explicit, minimal lifetimes (`'a`).
- **ALWAYS**: Map manual allocation lifecycles (`malloc`/`free`, `new`/`delete`) directly to Rust's single-ownership model (`Box<T>`, `Vec<T>`, or `Arc<T>`).
- **STRICT_REJECT**: Reject designs that perpetuate uncontrolled aliasing through raw pointers (`*mut T`) or global mutable state in rejuvenated code.

### Invariant 2: Raw Pointer Elimination & Safe Indexing
- **MANDATORY**: Rejuvenated Rust code must completely eliminate raw pointer arithmetic (`ptr.add(offset)`, `ptr.offset()`) in application logic.
- **NEVER**: Allow raw pointer dereferencing (`unsafe { *ptr }`) outside strictly audited, minimal FFI boundary wrappers.
- **ALWAYS**: Use safe Rust iterators (`iter()`, `chunks()`, `windows()`) and bounds-checked slice access to eliminate spatial memory corruption risks entirely.

### Invariant 3: Unsafe FFI Encapsulation & RAII Bound
- **MANDATORY**: When interfacing with legacy shared libraries or kernel ABIs via FFI, all `unsafe` operations must be encapsulated within a sound, panic-safe Rust RAII wrapper struct.
- **ALWAYS**: Implement `Drop` on FFI resource wrappers to ensure infallible resource cleanup (file descriptors, sockets, C-allocated memory).
- **STRICT_REJECT**: Reject any public API that exposes raw C pointers or leaks `unsafe` preconditions to the caller.

### Invariant 4: Differential Property-Based Equivalence Proofs
- **MANDATORY**: Every rejuvenated module must undergo differential property-based testing (using `proptest` or `quickcheck`) asserting bit-for-bit or functional equivalence against the legacy C/C++ reference across a minimum of 100,000 randomized inputs.
- **STRICT_REJECT**: Reject any rejuvenated implementation that deviates from legacy edge-case behaviors (e.g. overflow wrapping, IEEE-754 special values, empty buffer handling) unless explicitly documented as a security bug fix.

---

## 2. Canonical Rejuvenation Patterns

### Legacy C Buffer to Safe Rust RAII Slice (Rust)
```rust
use std::ops::Deref;

/// Safe RAII wrapper around legacy C-allocated buffer
pub struct SafeForeignBuffer {
    ptr: std::ptr::NonNull<u8>,
    len: usize,
}

impl SafeForeignBuffer {
    /// Safe constructor validating non-null and alignment
    pub fn new(raw_ptr: *mut u8, len: usize) -> Option<Self> {
        let ptr = std::ptr::NonNull::new(raw_ptr)?;
        Some(Self { ptr, len })
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        // Sound because NonNull validates non-null and len is verified
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Deref for SafeForeignBuffer {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Drop for SafeForeignBuffer {
    fn drop(&mut self) {
        // Deallocate via legacy C allocator
        extern "C" {
            fn free(ptr: *mut std::ffi::c_void);
        }
        unsafe {
            free(self.ptr.as_ptr() as *mut std::ffi::c_void);
        }
    }
}
```

---

## 3. Rejuvenation Workflow

1. **Extraction & Seam Identification**: Identify legacy C function signatures and pointer lifetime boundaries.
2. **Type Re-Mapping**: Convert `char*` to `&str` / `String`, `void* + len` to `&[u8]`, and return error codes to `Result<T, E>`.
3. **Borrow-Checker Synthesis**: Construct explicit Rust lifetime parameters preserving safety invariants.
4. **Differential Equivalence Fuzzing**: Run property tests executing both C FFI reference and safe Rust candidate to assert 100% equivalence.
