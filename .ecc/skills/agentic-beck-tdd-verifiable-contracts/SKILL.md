---
name: agentic-beck-tdd-verifiable-contracts
description: "Red-Green-Refactor cycle, test-first specifications, triangulation, isolation through test doubles, and regression test suites."
triggers: ["beck", "tdd", "test-driven-development", "red-green-refactor", "triangulation", "test-doubles", "verifiable-contracts"]
---

# agentic-beck-tdd-verifiable-contracts
> Based on **Test-Driven Development: By Example - Kent Beck**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Red-Green-Refactor Invariant: 1. Write a failing test (Red). 2. Write minimal code to pass the test (Green). 3. Clean up design without breaking tests (Refactor).**
2. **Triangulation: Generalize code logic only when you have two or more distinct examples/tests requiring that generalization.**
3. **Isolation: Tests must run independently in any order without shared mutable state or environmental dependencies.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Direct agents to always write the unit test FIRST. The agent must verify the test fails with the expected error before writing production code to pass it.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing production code before tests, leading to untestable designs or confirmation-biased tests.**
- **Writing tests that pass trivially without actually exercising the targeted failure mode.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "beck"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
