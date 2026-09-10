---
name: design-by-contract
description: "Design by Contract (Bertrand Meyer): preconditions, postconditions, class invariants, defensive boundary validation, and fail-fast invariant enforcement for self-verifying AI code generation."
triggers: ["design by contract", "dbc", "preconditions", "postconditions", "class invariants", "bertrand meyer", "defensive programming", "fail fast", "contract validation", "invariants"]
---

# Design by Contract (Bertrand Meyer)

This skill enforces Bertrand Meyer's *Design by Contract (DbC)* discipline to make software mathematically provable, self-verifying, and immune to cascading regression bugs.

## 1. The Contract Triad
Every function, method, and component interface must define three explicit elements:

1. **Preconditions (`require`)**:
   - Obligations that the **caller** must satisfy before invoking the function.
   - If a precondition is violated, the caller contains a defect; the function MUST abort immediately (fail-fast).
   - Example: `require(claim.amount >= 0.0, "Claim amount cannot be negative")`.
2. **Postconditions (`ensure`)**:
   - Guarantees that the **callee (function)** promises to deliver upon normal return.
   - If a postcondition fails, the function contains an internal bug.
   - Example: `ensure(result.total == result.jdf + result.saj + result.basic, "Fee breakdown must sum to total")`.
3. **Class / Aggregate Invariants (`invariant`)**:
   - System truths that must remain strictly TRUE throughout the entire life of an object.
   - Invariants must hold true before any public method is called, and must be restored before the method returns.
   - Example: `invariant(account.balance >= 0, "Account balance cannot be negative")`.

## 2. The Golden Rule of Exception Handling
- **Defensive Boundary Validation**: Validate and sanitize untrusted external inputs (HTTP bodies, CLI inputs, user forms) at the periphery, returning explicit domain error types (`Result::Err`).
- **Contract Assertions**: Internal domain logic between trusted internal components MUST use assertions/contracts. If an internal contract fails, terminate immediately rather than quietly continuing in a corrupted state.
- **Contract Non-Negotiation**: Never quietly catch and suppress contract violation errors.

## 3. Applying DbC to AI Vibe Coding
When prompting or reviewing AI code generation:
1. Specify explicit `@precondition` and `@postcondition` comments on all interfaces.
2. Instruct the AI to convert postconditions directly into property-based test assertions (`proptest` / `quickcheck`).
3. Ensure the AI does not write redundant defensive checks inside internal private helper methods when the public API contract already guaranteed the invariant.
