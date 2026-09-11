---
name: legacy-seams-characterization
description: "Working Effectively with Legacy Code (Michael Feathers): test-harness establishment, identifying seams, sensing and separation, characterization testing, and non-destructive sprout/wrap methods for unfamiliar or AI-generated code."
triggers: ["legacy code", "michael feathers", "seams", "characterization tests", "sprout method", "wrap method", "sensing and separation", "untested code", "working effectively with legacy code", "test harness"]
---

# Working Effectively with Legacy Code and Seams (Michael Feathers)

This skill equips the agent with Michael Feathers' disciplines for safely analyzing, modifying, and refactoring untested, unfamiliar, or freshly AI-generated black-box codebases.

## 1. The Operational Definition of Legacy Code
- **Legacy Code is simply code without automated tests**:
  - Without tests, code cannot be refactored safely, regardless of whether it was written 10 years ago in COBOL or 5 minutes ago by an LLM.
- **The Legacy Dilemma**:
  - To change code, you need tests in place. To put tests in place, you often have to change the code. Break this dilemma using non-invasive Seams.

## 2. Seams and Enabling Points
1. **Definition of a Seam**:
   - A place where you can alter behavior in your program without editing in that place.
2. **Types of Seams**:
   - **Object Seams**: Subclassing, interface injection, or trait replacement (common in OOP/Rust trait patterns).
   - **Link / Module Seams**: Diverting module resolution or linker symbols (e.g., mock crate features, test doubles).
   - **Preprocessing / Compile-Time Seams**: Conditional compilation (#[cfg(test)], macro overrides).
3. **Sensing vs. Separation**:
   - **Sensing**: Using seams to inspect values/side-effects computed inside otherwise opaque functions.
   - **Separation**: Using seams to decouple dependencies (e.g. databases, sockets) that prevent execution in a test harness.

## 3. Characterization Testing (Black-Box Behavioral Freezing)
1. **Purpose**:
   - Characterize the actual existing behavior of the system (including historical quirks and edge cases) rather than idealized theoretical behavior.
2. **Procedure**:
   - Write a test that asserts an arbitrary value (e.g., assert_eq!(result, "guess")).
   - Run the test harness; observe the actual returned output from the failure message.
   - Update the assertion to expect the real output.
   - Repeat until all major operational branches are pinned under test harness before attempting any refactoring.

## 4. Sprout and Wrap Interventions
1. **Sprout Method / Sprout Class**:
   - When adding a new feature to a tangled legacy function, write the new logic as a separate, pure, independently tested helper (Sprout), and call it from the original function.
2. **Wrap Method / Wrap Class (Decorator)**:
   - Wrap the legacy function with pre-processing and post-processing steps without altering the internal legacy implementation.
3. **Scratch Refactoring**:
   - Refactor freely on a throwaway branch to understand the problem domain, then discard the scratch code and re-apply changes methodically under tests.
