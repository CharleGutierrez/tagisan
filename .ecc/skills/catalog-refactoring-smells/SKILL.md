---
name: catalog-refactoring-smells
description: "Refactoring and Code Smells Catalog (Martin Fowler): deterministic detection of architectural code smells, behavioral preservation, and atomic AST transformations."
triggers: ["refactoring", "martin fowler", "code smells", "feature envy", "primitive obsession", "data clumps", "shotgun surgery", "replace conditional with polymorphism", "extract function", "behavior preserving"]
---

# Code Smells and Atomic Refactoring Catalog (Martin Fowler)

This skill equips the agent with Martin Fowler's canonical catalog of code smells and behavior-preserving atomic refactorings to clean, simplify, and restructure existing code with zero behavioral regression.

## 1. Diagnostic Code Smells
1. **Primitive Obsession**:
   - Overusing raw strings, integers, or tuples to represent rich domain concepts (e.g., email: String, currency: String, amount: i64). Replace with dedicated Value Types with built-in validation.
2. **Feature Envy**:
   - A function that accesses data and methods of another object more than its own. Move the function to the data it envies.
3. **Data Clumps**:
   - Groups of 3 or more fields frequently passed together across functions (e.g. x, y, width, height or start_date, end_date). Introduce a Parameter Object or Struct.
4. **Divergent Change vs. Shotgun Surgery**:
   - **Divergent Change**: One module is repeatedly changed in different ways for different reasons (violates Single Responsibility). Split into separate modules.
   - **Shotgun Surgery**: One small change forces dozens of minor edits across many different files. Consolidate into a single module.
5. **Speculative Generality**:
   - Unused hooks, abstract base classes, and generic type parameters added 'just in case'. Delete them; adhere strictly to YAGNI.

## 2. Core Behavior-Preserving Transformations
1. **Extract Function / Method**:
   - Turn a block of code with high cognitive load into a cleanly named helper function that documents its intent.
2. **Replace Temp with Query**:
   - Replace temporary state variables with pure deterministic calculation queries to reduce mutable state.
3. **Introduce Parameter Object**:
   - Bundle recurring argument clusters into an immutable configuration struct.
4. **Replace Conditional with Polymorphism / Pattern Matching**:
   - Replace sprawling switch/if-else ladders with dynamic dispatch, traits, or exhaustive enum matching.
5. **Decompose Conditional**:
   - Extract complicated boolean conditions into well-named predicate functions (e.g. if should_retry_request(...)).

## 3. The Refactoring Rhythm
- Refactor in micro-steps: make one atomic structural transformation, run the test suite, ensure green, commit or proceed.
- Never add new features and refactor at the exact same moment; separate structural commits from behavioral commits.
