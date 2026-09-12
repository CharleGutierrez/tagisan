---
name: arch-love-linux-kernel
description: "Operating system internals: Process scheduling, Virtual File System (VFS), Page Cache, non-blocking I/O (epoll, io_uring), memory mapping, and interrupt handling."
triggers: ["robert-love", "linux-kernel", "epoll", "io-uring", "page-cache", "vfs", "memory-mapping", "non-blocking-io"]
---

# arch-love-linux-kernel
> Based on **Linux Kernel Development - Robert Love**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: High-throughput network and disk I/O must utilize asynchronous event-driven readiness polling (epoll/kqueue) or completion queues (io_uring).**
2. **ALWAYS: Leverage the OS page cache via memory-mapped files (mmap) for read-intensive file databases and read-only catalogs.**
3. **NEVER: Spawn an unbounded OS thread per connection; bound worker thread pools to available hardware concurrency.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design server runtimes using event loops (epoll/io_uring) with bounded worker pools. Use mmap for cold startup acceleration. Eliminate redundant user/kernel data copying.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Spawning 10,000 OS threads for 10,000 connections (thread stack OOM).**
- **Repeatedly reading small disk chunks with synchronous read() syscalls.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-love-linux-kernel"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
