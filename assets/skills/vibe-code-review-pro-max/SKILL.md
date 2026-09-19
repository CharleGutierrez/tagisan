---
name: vibe-code-review-pro-max
description: Autonomous Master Engine for the Top 100 Code and Program Review Books for Vibe Coders. Enforces 10 canonical review domains (Peer Review, Refactoring Smells, Craftsmanship, AppSec Auditing, System Architecture, Concurrency Invariants, Property Verification, Low-Level Systems, Language Idioms, and AI Steering). Scrutinizes AI-generated code for shallow module bloat, subtle concurrency bugs, injection vectors, and missing invariant cages. Triggers: vibe-review, code-review, pr-review, ousterhout, fowler, dowd, kleppmann, goetz, code-smells, refactoring, security-audit, invariants, test-cages, deep-modules.
version: 1.0.0
tags:
  - vibe-review
  - code-review
  - security-audit
  - refactoring
  - deep-modules
  - concurrency
  - property-testing
  - static-analysis
compatibility: ">=0.2.0"
---

# Vibe Code Review Pro Max: The Sovereign 100-Book Inspection Engine

## Purpose & Scope
In the AI era, code generation is cheap, instantaneous, and ubiquitous. However, uninspected AI code routinely introduces:
1. **Shallow Module Anti-Patterns & Cognitive Bloat** (Ousterhout)
2. **Hidden Security Vulnerabilities & Insecure Defaults** (Dowd, Zalewski)
3. **Silent Concurrency Hazards, Deadlocks & Memory Races** (Goetz, McKenney)
4. **Untested Invariant Drifts & Brittle Vanity Tests** (Khorikov, Hébert)

The `vibe-code-review-pro-max` skill integrates the wisdom of the **Top 100 Code and Program Review Books** into Tagisan (`tgs review`), providing both human developers and autonomous AI swarms with an automated 5-layer verification cage.

---

## 1. The 10 Macro-Review Clusters (100 Books)

| Cluster Code | Domain Name | Books | Canonical Authors | Primary Review Invariant |
|---|---|---|---|---|
| `VBR-01` | **Peer Code Review & Formal Inspection** | 10 | Wiegers, Winters (Google), Cohen, Fagan, Gilb | Atomic PR sizing (<400 lines), asynchronous critique, and defect density tracking |
| `VBR-02` | **Code Smells, Refactoring & Readability** | 10 | Ousterhout, Fowler, Feathers, Boswell, Kerievsky | Deep vs Shallow modules, zero dead wrappers, legacy seams around unverified code |
| `VBR-03` | **Software Construction & Craftsmanship** | 10 | McConnell, Thomas & Hunt, Kernighan & Pike | Defensive programming, broken window repair, clean parameter boundaries |
| `VBR-04` | **Security Auditing & Defensive Coding** | 10 | Dowd, Howard & LeBlanc, Zalewski, Shostack | Trust boundary enforcement, STRIDE threat modeling, domain-driven type security |
| `VBR-05` | **Architecture & System Design Evaluation** | 10 | Kleppmann, Richards & Ford, Evans (DDD), Nygard | Distributed state isolation, bounded contexts, circuit breakers, and fault tolerance |
| `VBR-06` | **Concurrency, Multithreading & Invariants** | 10 | Goetz, McKenney, Herlihy & Shavit, Lamport | Happens-before memory models, thread-safe publication, lockless queue correctness |
| `VBR-07` | **Testing, Verification & Test Smells** | 10 | Khorikov, Meszaros, Beck (TDD), Hébert | Invariant testing, eliminating test smells, property-based input generators |
| `VBR-08` | **Debugging, Profiling & Systems Performance** | 10 | Brendan Gregg, Bryant & O'Hallaron, Kerrisk | Syscall latency tracing, CPU cache alignment, memory leak elimination |
| `VBR-09` | **Language-Specific Review Bibles** | 10 | Bloch (Java), Meyers (C++), Blandy (Rust), Slatkin | Idiomatic ownership, RAII, type narrowing, and memory safety invariants |
| `VBR-10` | **Mindset, Systems & AI-Native Review** | 10 | Brooks, Google SRE, Geewax, Chip Huyen | Conceptual integrity, SLO error budgeting, API idempotency, and deterministic agent cages |
| **TOTAL** | **10 Clusters** | **100** | **Global Software Literature** | **Deterministic Verification, Zero Hallucination, Sovereign Code Quality** |

---

## 2. The Vibe Coder's 5-Layer Inspection Checklist

```
Layer 1: Correctness & Semantic Invariants
  ↳ Does the code actually solve the user's objective without hallucinating unstated assumptions?
  ↳ Are return values properly typed (Result/Option), with zero silent failure swallowing?

Layer 2: Edge Cases & Boundary Traps
  ↳ Check off-by-one indices (.. vs ..=).
  ↳ Verify all files, sockets, connections, and mutexes are properly closed or dropped.
  ↳ Inspect async logic for unawaited futures or blocking calls inside event loops.

Layer 3: Security & Trust Boundaries
  ↳ Never permit untrusted data to concatenate into SQL queries, system commands, or file paths.
  ↳ Forbid wildcard CORS origins (*) or disabling TLS verification in production paths.
  ↳ Enforce strong domain types rather than primitive string passing.

Layer 4: Architectural Cohesion & Simplicity
  ↳ Eliminate "Shallow Modules" (functions that take more lines of documentation than actual logic).
  ↳ Ban dependency bloat: reject 20MB third-party imports for 5-line string manipulation tasks.

Layer 5: The Verification Cage
  ↳ Require automated property-based tests or rigorous edge-case unit tests.
  ↳ Run static linters and compiler typechecks before approving any merge.
```

---

## 3. CLI & Tool Commands

- `tgs review list [--cluster <cluster>] [--limit <n>]`: Browse the 100-book review catalog.
- `tgs review search <query>`: Fast inverted search across all 100 review book heuristics.
- `tgs review get <id>`: View the complete review invariants for a specific book.
- `tgs review audit <file>`: Run the automated static code analyzer against the 5-layer review heuristics.
- `tgs review playbook [--stage <1..5>]`: Display the curated 5-stage vibe coder reading path.
- `tgs review export [--format markdown|json]`: Export the catalog to standard formats.
