---
name: arch-bryant-computer-systems
description: "Hardware-aware systems engineering: CPU memory hierarchies (L1/L2/L3), cache lines, spatial/temporal locality, branch prediction, virtual memory, and process linking."
triggers: ["csapp", "mechanical-sympathy", "cache-lines", "spatial-locality", "branch-prediction", "virtual-memory", "hardware-sympathy"]
---

# arch-bryant-computer-systems
> Based on **Computer Systems: A Programmer's Perspective (CS:APP) - Randal E. Bryant & David R. O'Hallaron**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Hot-path data structures must be laid out in contiguous memory (Arrays/Vecs) to maximize CPU L1/L2 cache hit ratios and prefetching.**
2. **ALWAYS: Prevent cache line bouncing (false sharing) by padding atomic variables across distinct 64-byte cache line boundaries.**
3. **NEVER: Traverse multidimensional arrays in column-major order in row-major memory representations (stride-1 access invariant).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure performance-critical data structures for cache locality. Align hot structs to 64-byte boundaries. Eliminate pointer chasing inside tight inner execution loops.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Linked lists in hot paths causing continuous L1 cache misses.**
- **Concurrent threads updating adjacent atomic variables on the same 64-byte cache line.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-bryant-computer-systems"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
