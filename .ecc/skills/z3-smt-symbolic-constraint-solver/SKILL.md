---
name: z3-smt-symbolic-constraint-solver
description: "Z3 / CVC5 SMT-LIB2 symbolic execution, constraint solving, bitvector satisfiability, path reachability, and equivalence proofs."
version: 0.2.0
tags:
  - smt
  - z3
  - cvc5
  - smt-lib2
  - symbolic-execution
  - bitvector
  - formal-methods
  - satisfiability
  - equivalence-proof
compatibility: ">=0.2.0"
triggers:
  - "z3"
  - "cvc5"
  - "smt-lib"
  - "smt2"
  - "symbolic execution"
  - "constraint solver"
  - "bitvector"
  - "path reachability"
  - "equivalence proof"
  - "sat"
  - "unsat"
---

# Z3 & CVC5 SMT-LIB2 Symbolic Constraint Solver Skill

The `z3-smt-symbolic-constraint-solver` skill provides automated formal verification, bitvector satisfiability, symbolic execution path reachability, and expression equivalence proofs utilizing the SMT-LIB2 standard (supported by Z3, CVC5, and Boolector).

## Core Capabilities

1. **SMT-LIB2 Standardized Script Synthesis**: Generates syntactically conformant, highly optimized SMT-LIB2 bitvector (`QF_BV`) and array (`QF_ABV`) formulas with variable declarations, assertions, and solver directives.
2. **Exact Bitvector Satisfiability & Model Extraction**: Solves non-linear and linear bitvector constraints, extracting concrete variable assignments that satisfy complex systems of equations or isolating unsatisfiable cores.
3. **Semantic Equivalence Verification via Negation UNSAT Proof**: Verifies algebraic identities, bit-twiddling optimizations, and compiler transformations by asserting the negation of equivalence `(assert (distinct exprA exprB))` and proving mathematical unsatisfiability (`unsat`).
4. **Symbolic Range & Path Reachability Analysis**: Proves that critical program points cannot trigger out-of-bounds array access, integer overflow, or division by zero under all valid symbolic inputs.

## Strict Operational Invariants

- **ALWAYS**:
  - Format all SMT assertions strictly under standard SMT-LIB2 grammar (`(set-logic QF_BV)` or `(set-logic QF_ABV)` for bitvectors and memory arrays).
  - Check satisfiability with `(check-sat)` and extract concrete counterexample variable assignments with `(get-model)` on SAT results.
  - Prove semantic equivalence of expressions A and B by formulating negation: assert `(assert (distinct expr_a expr_b))` and verify that the solver reports `unsat`.
  - Declare explicit bit-widths for all symbolic variables (e.g. `(_ BitVec 32)`, `(_ BitVec 64)`) to eliminate signedness, sign-extension, and truncation ambiguities.

- **NEVER**:
  - NEVER rely on unbounded integer theories (`QF_LIA`, `QF_NIA`) when modeling hardware registers or fixed-width integer arithmetic that can overflow.
  - NEVER treat an `unknown` or `timeout` solver response as a proof of unsatisfiability or correctness.
  - NEVER discard solver model counterexamples; always map concrete variable valuations back to offending inputs.
  - NEVER omit range assertions and pre-conditions on symbolic inputs when checking path reachability.

- **MANDATORY**:
  - MANDATORY assert non-zero divisors for symbolic division and modulo operations (`bvudiv`, `bvsdiv`, `bvurem`, `bvsrem`) to prevent div-by-zero undefined behavior.
  - MANDATORY log SMT-LIB script generation metrics including variable counts, asserted constraint count, logic dialect, and solver execution time.
  - MANDATORY verify that bitvector shifts (`bvshl`, `bvlshr`, `bvashr`) do not exceed the bit-width of the target operand.

- **STRICT_REJECT**:
  - STRICT_REJECT any equivalence claim that was derived from a SAT result (SAT means a counterexample exists, proving expressions are NOT equivalent).
  - STRICT_REJECT unconstrained symbolic arrays without finite bounds when checking buffer boundaries.
  - STRICT_REJECT any solver output that suppresses syntax or sort mismatch errors.
