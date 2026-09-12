---
name: arch-herlihy-multiprocessor-programming
description: "Multi-core concurrency theory: Linearizability, lock-free & wait-free data structures, Compare-And-Swap (CAS), ABA problem, hazard pointers, and transactional memory."
triggers: ["herlihy-shavit", "multiprocessor-programming", "lock-free", "wait-free", "compare-and-swap", "aba-problem", "hazard-pointers", "atomic-concurrency"]
---

# arch-herlihy-multiprocessor-programming
> Based on **The Art of Multiprocessor Programming - Maurice Herlihy & Nir Shavit**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Shared concurrent state must prefer lock-free single-writer or bounded message-passing channels over coarse-grained global mutex locks.**
2. **ALWAYS: Protect Compare-And-Swap (CAS) pointers from the ABA problem using generation counters or hazard pointers.**
3. **NEVER: Hold a mutex lock across an asynchronous `.await` or blocking I/O boundary.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement multi-threaded coordination using lock-free primitives or bounded MPSC queues. Verify linearizability of concurrent data structures. Eliminate lock convoying.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Holding locks while waiting for network responses.**
- **Assuming increment operators (i++) are thread-safe without atomic memory ordering.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-herlihy-multiprocessor-programming"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
