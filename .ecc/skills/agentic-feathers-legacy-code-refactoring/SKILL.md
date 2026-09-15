---
name: agentic-feathers-legacy-code-refactoring
description: "Legacy code definition (code without tests), sensing and separation pins, sprout/wrap method, characterization tests, and breaking dependencies."
triggers: ["feathers", "legacy-code", "characterization-tests", "sprout-method", "wrap-method", "breaking-dependencies", "seams"]
---

# agentic-feathers-legacy-code-refactoring
> Based on **Working Effectively with Legacy Code - Michael C. Feathers**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Definition of Legacy Code: Code without unit tests. Tests are a safety harness allowing rapid, fearless modification without regressions.**
2. **Characterization Tests: Tests that document and preserve the existing actual behavior of a legacy system before attempting refactoring.**
3. **Seams: A place where you can alter behavior in a program without editing in that place (e.g. object seams, link seams).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Before modifying legacy code, write characterization tests to lock down current behavior. Use Sprout/Wrap methods to add new features safely.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Refactoring complex legacy code without first establishing automated regression tests.**
- **Assuming undocumented legacy behavior is a bug and removing it, breaking downstream clients.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "feathers"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
