---
name: rust-idiomatic-hygiene
description: Idiomatic Rust patterns, RAII memory safety, thiserror error trees, and zero-copy Cow abstractions
---

# Rust Idiomatic Hygiene & Memory Architecture

## Core Guidelines
1. **Error Architecture**: Represent domain failure modes with explicit enum variants derived via `thiserror::Error`. Never discard errors with `.unwrap()` in library code.
2. **Zero-Copy Ergonomics**: Use `std::borrow::Cow<'a, str>` or `&str` when data transformation is optional.
3. **RAII & Clean Teardown**: Leverage the `Drop` trait to guarantee deterministic cleanup of file handles, temporary directories, worktrees, and sockets.
4. **Type-Driven Design**: Make illegal states unrepresentable using Rust's algebraic data types (`enum`, `struct`, newtype pattern).
