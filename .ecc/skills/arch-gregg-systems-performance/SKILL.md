---
name: arch-gregg-systems-performance
description: "Performance engineering methodology: USE Method (Utilization, Saturation, Errors), Off-CPU latency analysis, Flame Graphs, eBPF tracing, and operating system bottlenecks."
triggers: ["brendan-gregg", "systems-performance", "use-method", "flame-graphs", "ebpf-profiling", "off-cpu-analysis", "saturation-metrics"]
---

# arch-gregg-systems-performance
> Based on **Systems Performance: Enterprise and the Cloud - Brendan Gregg**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Instrument system bottlenecks using the USE method: track Utilization (%), Saturation (queue depth), and Errors for every physical and logical resource.**
2. **ALWAYS: Profile latency with Off-CPU analysis and Flame Graphs to distinguish CPU execution time from lock/I/O wait time.**
3. **NEVER: Benchmark or optimize performance without an isolated baseline, fixed warmup period, and statistical percentiles (p50, p99, p99.9).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Instrument services with USE metrics for CPU, memory, disks, and network sockets. Generate Flame Graphs for bottleneck identification. Eliminate off-CPU thread blocking.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying on average latency metrics while ignoring p99 tail latency spikes.**
- **Optimizing CPU instructions when the real bottleneck is lock contention or disk I/O wait.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-gregg-systems-performance"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
