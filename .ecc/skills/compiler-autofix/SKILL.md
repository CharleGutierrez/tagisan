---
name: compiler-autofix
description: Self-healing compiler diagnosis and surgical AST patch synthesis for Rust, TypeScript, Python, and Go.
---

# Compiler Autofix Skill

## Purpose
Automate compiler error diagnosis, test failure triage, and surgical AST patch application across polyglot software projects using Tagisan's `tgs autofix` engine.

## Invariant Rules
1. **Zero Production Panics**: Never call `.unwrap()` or panic on unexpected compiler outputs or syntax spans. Use safe UTF-8 character boundary adjustments and robust default fallbacks.
2. **Descending Order Patch Application**: When applying multiple span patches to a single file, sort replacements descending by `(line, column)` so earlier coordinates remain valid.
3. **Atomic Backups**: Always create a `.bak` snapshot prior to writing AST patches to disk.
4. **Iterative Verification**: Continue the diagnose -> patch -> verify loop until all diagnostics are resolved or `max_attempts` is reached.

## Supported Ecosystems & Diagnostics
- **Rust (`cargo check`, `cargo test`)**:
  - `unused_variables`: Automatically prefix with underscore or apply compiler suggested replacement.
  - `unused_mut`: Remove `mut` keyword.
  - Missing semicolons, mismatched types, unresolved imports.
- **TypeScript (`tsc`)**:
  - `TS6133` (unused declarations), `TS2322` (type assignment), missing imports.
- **Python (`py_compile`, `pytest`)**:
  - Missing `:` on control flow and function definitions (`def`, `class`, `if`, `for`, `while`, `try`, `except`).
  - Traceback and assertion diagnostics.
- **Go (`go vet`, `go build`)**:
  - Unused variables and imports, syntax errors.

## CLI Invocation
```bash
# Heal current directory
tgs autofix .

# Heal with test diagnostics included
tgs autofix . --test

# Perform dry run without modifying files
tgs autofix . --dry-run

# Limit healing attempts
tgs autofix . --max-attempts 3
```
