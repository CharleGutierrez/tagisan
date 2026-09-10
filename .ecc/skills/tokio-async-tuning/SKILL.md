---
name: tokio-async-tuning
description: Tokio async concurrency, lock contention avoidance, bounded channels, and JoinSet task scheduling
---

# Tokio Async & Concurrency Optimization

## Core Concurrency Protocols
1. **Channel Bounding**: Always use bounded channels (`tokio::sync::mpsc::channel(N)`); never unbounded in production hot paths.
2. **Lock Contention Minimization**: Never hold a `MutexGuard` across an `.await` boundary. Use message passing or atomic primitives (`AtomicBool`, `AtomicUsize`) for scalars.
3. **Task Orchestration**: Prefer `tokio::task::JoinSet` over loose `tokio::spawn` calls to ensure structured concurrency, cancellation propagation, and clean resource cleanup.
4. **Blocking Operations**: Offload heavy synchronous computations, CPU-bound parsing, or blocking filesystem I/O to `tokio::task::spawn_blocking`.
