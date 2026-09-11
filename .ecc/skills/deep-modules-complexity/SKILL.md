---
name: deep-modules-complexity
description: "A Philosophy of Software Design (John Ousterhout): deep vs shallow modules, narrow interfaces hiding complex implementations, information hiding, defining errors out of existence, and eliminating pass-through abstractions."
triggers: ["deep modules", "ousterhout", "philosophy of software design", "shallow modules", "information hiding", "define errors out of existence", "cognitive load", "interface complexity", "pass through methods", "strategic programming"]
---

# Deep Modules and Complexity Control (John Ousterhout)

This skill equips the agent with the design philosophy from John Ousterhout's *A Philosophy of Software Design*, specifically aimed at preventing AI-generated architectural rot, shallow boilerplate, and cognitive overload.

## 1. Deep vs. Shallow Modules
1. **The Cost/Benefit of an Interface**:
   - The interface is the cost a module imposes on the rest of the system (cognitive load, API surface, maintenance contract).
   - The functionality is the benefit the module provides.
   - **Deep Module (Target)**: A simple, narrow interface that conceals extensive, sophisticated internal implementation (e.g., standard Unix I/O read/write, garbage collectors, OS threads).
   - **Shallow Module (Anti-pattern)**: A complex or wide interface that hides very little actual logic (e.g., 5-line wrapper classes that merely forward arguments). Eliminate shallow wrappers.

2. **Information Hiding vs. Information Leakage**:
   - **Information Hiding**: Internal algorithms, concurrency synchronization, data formats, and caching policies must never leak through function signatures or return types.
   - **Information Leakage (Anti-pattern)**: Requiring callers to understand sequencing rules (e.g. calling init, prepare, validate before execute). Combine them into a single coherent call.

## 2. Defining Errors Out of Existence
1. **Exception Proliferation**:
   - Exceptions increase cognitive load exponentially. Every exception requires error-handling logic at call sites.
2. **Techniques to Eliminate Errors**:
   - **Subsumption**: Redefine semantics so the error condition is simply a valid, well-defined subset of normal behavior (e.g., deleting a non-existent file or substring returns success/no-op instead of throwing).
   - **Masking**: Handle errors internally at low levels (e.g. network retries with backoff) rather than bubbling trivial transient errors to the caller.
   - **Aggregation**: Handle multiple potential errors at a single high-level boundary rather than peppering individual functions with try-catch blocks.

## 3. Eliminating Pass-Through Anti-Patterns
1. **Pass-Through Methods**:
   - If method A calls method B with virtually identical signatures and no transformation, module boundaries are drawn incorrectly. Merge or flatten them.
2. **Pass-Through Variables**:
   - Do not pass an argument through 5 intermediate function layers just to reach the 6th layer. Use context objects or bind dependencies at construction time.

## 4. Strategic vs. Tactical Programming
1. **The Tactical Trap**:
   - Tactical programming focuses solely on getting the immediate feature or prompt working as fast as possible, incurring accumulated complexity ("death by a thousand cuts").
2. **Strategic Discipline**:
   - Invest 10-20% of engineering effort on structural design, clean separation of concerns, and documentation of non-obvious invariants.
   - Design modules so that future changes become easier, not harder.
