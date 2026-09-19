---
name: memory-governor-anti-freeze
description: Autonomous Host Memory Governor & Anti-Freeze Shield preventing Linux desktop freezes and OOM kills on 8GB workstations.
tools:
  - read_file
  - run_command
  - calculator
---

# Host Memory Governor & Anti-Freeze Shield (TGS Skill Guide)

## Overview
The TGS Host Memory Governor actively monitors Linux kernel telemetry (`/proc/meminfo` and `/proc/pressure/memory`), manages process heap fragmentation via `libc::malloc_trim(0)`, dynamically clamps worker concurrency, and preserves system responsiveness on 8GB laptops.

## The 3 Pressure Tiers
- **Green Tier (> 25% RAM Available)**: Healthy execution with full concurrency and unrestricted caching.
- **Yellow Tier (15% - 25% RAM Available)**: Dynamic concurrency braking clamps Tokio and Rayon worker pools down to 1-2 threads, shrinking vector embedding batches.
- **Red Tier (< 15% RAM Available or Swap > 85%)**: Emergency Freeze Shield:
  - Executes `libc::malloc_trim(0)` to immediately return unmapped glibc heap pages to the OS kernel.
  - Flushes in-memory semantic embedding caches to disk.
  - Pauses new subagent spawning until host RAM recovers.
  - Enforces `CARGO_BUILD_JOBS=1` and single-threaded compilation.

## REPL Commands
- `/mem status`: Live RAM/Swap telemetry, pressure tiers, and concurrency limits
- `/mem trim`: Forcefully return unused heap arenas to Linux kernel via malloc_trim(0)
- `/mem guard`: Display anti-freeze background guard parameters and quotas
- `/mem help`: Show manual
