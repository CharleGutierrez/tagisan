---
name: post-quantum-constant-time-auditor
description: Post-quantum cryptography and constant-time execution audit. Eliminates timing side-channels, strictly forbids conditional branches or memory lookups on secret data, enforces zeroization of sensitive buffers, and audits PQC algorithms (Kyber/Dilithium).
version: 1.0.0
tags:
  - constant-time
  - cryptography
  - post-quantum
  - timing-attacks
  - side-channel
  - zeroize
  - subtle
  - pqc
triggers:
  - constant-time
  - timing-leak
  - post-quantum
  - pqc
  - kyber
  - dilithium
  - zeroize
  - secret-branching
  - side-channel-audit
compatibility: ">=0.2.0"
---

# Post-Quantum & Constant-Time Cryptographic Auditor: Side-Channel Elimination

## Purpose & Scope
Cryptographic implementations—particularly post-quantum algorithms (ML-KEM / Kyber, ML-DSA / Dilithium, SLH-DSA / SPHINCS+) and classical asymmetric primitives—are catastrophically vulnerable to side-channel timing attacks. Variable-time arithmetic, secret-dependent conditional branches (`if secret == 0`), secret-dependent memory indexing (`table[secret]`), and residual secret material on the stack/heap allow attackers to extract private keys across network boundaries or co-located VMs.

This skill equips autonomous systems with rigorous constant-time verification protocols. It guarantees that all code paths manipulating secret keys execute in constant time independent of secret data values, mandates zeroization on drop, and validates PQC lattice polynomial operations.

---

## 1. Operational Invariants (Constant-Time Verification Protocol)

### Invariant 1: Elimination of Secret-Dependent Conditional Branching
- **MANDATORY**: Secrets and sensitive cryptographic variables (private keys, shared secrets, nonces, plaintext messages) must NEVER determine control flow. No `if`, `else`, `match`, `while`, or ternary expressions may depend on secret values.
- **ALWAYS**: Use constant-time conditional selection (`subtle::ConditionallySelectable`, `subtle::Choice`) or branchless bitwise masking (`mux = (mask & a) | (~mask & b)`) to choose between secret values.
- **STRICT_REJECT**: Reject any implementation containing early-exit equality comparisons (`==` or `!=`) on secret buffers. Equality checks must use constant-time comparisons (`subtle::ConstantTimeEq`).

### Invariant 2: Elimination of Secret-Dependent Memory Indexing
- **MANDATORY**: Array lookups and memory accesses indexed by secret variables (`table[secret]`) are strictly forbidden. Cache-timing attacks (Flush+Reload, Prime+Probe) can reconstruct secret indices by measuring memory access latency.
- **ALWAYS**: Perform constant-time table lookups using linear scans with branchless multiplexing or bit-sliced implementations where all elements of the table are accessed sequentially regardless of the secret index.
- **STRICT_REJECT**: Reject secret-dependent s-box lookups or non-constant-time permutation table reads.

### Invariant 3: Cryptographic Memory Scrubbing & Zeroization
- **MANDATORY**: All memory buffers, stack arrays, and heap allocations holding sensitive keys or intermediate ephemeral states must be scrubbed immediately upon deallocation or function return.
- **ALWAYS**: Derive or implement `zeroize::Zeroize` and `zeroize::ZeroizeOnDrop` for structs holding sensitive cryptographic material.
- **NEVER**: Rely on standard variable re-assignment or plain `memset(0)` to scrub secrets, as optimizing compilers eliminate such writes as dead-store elimination (DSE). Use volatile memory writes (`core::ptr::write_volatile`) or the `zeroize` crate.

### Invariant 4: Post-Quantum Lattice & Noise Invariants
- **MANDATORY**: Post-quantum lattice implementations (Kyber / Dilithium) must execute polynomial Number Theoretic Transform (NTT), modular reduction (Barrett or Montgomery reduction), and rejection sampling with strictly uniform timing.
- **NEVER**: Use variable-time integer division (`/`) or modulo (`%`) on secret polynomials, as hardware divider execution latency varies with operand bit patterns.
- **STRICT_REJECT**: Reject PQC Gaussian or centered binomial noise sampling routines that terminate early on secret condition flags.

---

## 2. Canonical Constant-Time Patterns

### Constant-Time Conditional Swap & Secret Buffer Hygiene (Rust)
```rust
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretKeyMaterial {
    pub key: [u8; 32],
    pub scalar: [u8; 32],
}

impl SecretKeyMaterial {
    #[inline(never)]
    pub fn verify_token_constant_time(&self, candidate: &[u8; 32]) -> bool {
        // subtle ConstantTimeEq executes in constant cycles regardless of match offset
        let is_equal: Choice = self.key.ct_eq(candidate);
        is_equal.into()
    }
}

#[inline(always)]
pub fn constant_time_select(condition: Choice, true_val: u32, false_val: u32) -> u32 {
    let mut result = false_val;
    result.conditional_assign(&true_val, condition);
    result
}

#[inline(always)]
pub fn constant_time_lookup_32(table: &[u32; 16], secret_idx: u8) -> u32 {
    let mut result: u32 = 0;
    for i in 0..16 {
        let match_choice = (secret_idx as usize).ct_eq(&i);
        result.conditional_assign(&table[i], match_choice);
    }
    result
}
```

---

## 3. Tool Invocations

Use `constant_time_auditor` to detect timing side-channels and verify buffer scrubbing:
- `constant_time_auditor(action: "audit_timing_leaks", code: "if secret_key[i] == candidate[i] { ... }", sensitive_variables: ["secret_key", "scalar"])`
- `constant_time_auditor(action: "verify_secret_zeroization", code: "struct Key { buf: [u8; 32] }", sensitive_variables: ["buf"])`
