---
name: modular-coupling-cohesion
description: "Structured Systems Design (Meilir Page-Jones): module cohesion hierarchy, coupling reduction, fan-in/fan-out constraints, and transform/transaction factoring for AI-generated code."
triggers: ["cohesion", "coupling", "modular design", "page-jones", "structured design", "functional cohesion", "data coupling", "fan-in", "fan-out", "refactor modules"]
---

# Modular Systems Design: Cohesion & Coupling (Meilir Page-Jones)

This skill enforces the architectural rules of Meilir Page-Jones' *The Practical Guide to Structured Systems Design* to guarantee high cohesion, loose coupling, and maintainable modular architecture.

## 1. The 7 Levels of Module Cohesion (Target: Functional Cohesion)
When designing functions, structs, or classes, evaluate their cohesion level:
1. **Functional Cohesion (HIGHEST - Enforce This)**:
   - The module performs exactly one problem-related task (e.g., `calculate_filing_fee(claim) -> FeeSummary`).
   - Every line of code contributes directly to that single objective.
2. **Sequential Cohesion (Acceptable)**:
   - Elements are grouped because the output of one step serves as direct input to the next step.
3. **Communicational Cohesion (Acceptable with care)**:
   - Functions grouped because they operate on the exact same input data structure.
4. **Procedural Cohesion (AVOID)**:
   - Functions grouped solely because they execute in a specific order.
5. **Temporal Cohesion (AVOID)**:
   - Grouping operations solely because they occur at the same time (e.g., `init_all_services()`).
6. **Logical Cohesion (DANGEROUS)**:
   - A single module containing a multi-branch switch/flag doing unrelated things based on a parameter (e.g., `do_task(action: string)`).
7. **Coincidental Cohesion (FORBIDDEN)**:
   - Arbitrary grouping of unrelated functions (e.g., `utils.ts`, `helpers.rs`). Split immediately into functional modules.

## 2. The 5 Levels of Module Coupling (Target: Data Coupling)
1. **Data Coupling (BEST - Enforce This)**:
   - Modules communicate solely by passing discrete, strictly typed data arguments (`user_id: Uuid, amount: Decimal`).
2. **Stamp Coupling (Acceptable with pruning)**:
   - Modules pass an entire composite structure when only a subset of fields is needed. Prune to pass only required attributes.
3. **Control Coupling (AVOID)**:
   - One module passes control flags (e.g., `is_admin`, `do_print`, `mode`) that dictate the internal control flow of another module. Split into distinct specialized functions instead.
4. **Common Coupling (DANGEROUS)**:
   - Modules communicate via global shared mutable memory or shared mutable databases without encapsulation.
5. **Content Coupling (FORBIDDEN)**:
   - One module directly accesses or mutates the internal private state of another module.

## 3. Structural Factoring Heuristics
- **Fan-Out Rule**: A single function or coordinator must coordinate at most 7 subordinate helpers (Miller's $7 \pm 2$ limit).
- **Fan-In Rule**: High fan-in is encouraged—utility, validation, and mathematical routines should be reused across multiple call sites.
- **Transform-Centered Design**: Separate modules cleanly into:
  - Input Afferent Branch (validating and parsing input).
  - Central Transform (pure business domain logic).
  - Output Efferent Branch (formatting and emitting results).
