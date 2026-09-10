---
name: coding-standards
description: Strict code hygiene, cyclomatic complexity limits, zero dead code, and maintainability enforcement
---

# Engineering Coding Standards

## Standards Checklist
1. Cyclomatic Complexity: Keep function complexity under 10; decompose complex branched logic into small, testable helpers.
2. Zero Dead Code: Eliminate unused variables, dead imports, and obsolete comments.
3. Consistent Formatting: Follow standard linters and formatters (`cargo fmt`, `cargo clippy`, `prettier`).
4. Self-Documenting Code: Name identifiers for intent rather than implementation mechanics.
