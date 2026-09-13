---
name: xdp-dpdk-kernel-bypass-firewall
description: "High-performance line-rate packet processing, eXpress Data Path (XDP), DPDK userspace kernel-bypass driver, DDoS mitigation, and zero-allocation packet filtering."
version: 0.2.0
tags:
  - networking
  - xdp
  - dpdk
  - kernel-bypass
  - bpf
  - firewall
  - line-rate
  - ddos
compatibility: ">=0.2.0"
triggers:
  - "xdp"
  - "dpdk"
  - "kernel bypass"
  - "line speed"
  - "line rate"
  - "100gbe"
  - "packet filter"
  - "xdp_drop"
  - "af_xdp"
  - "zero copy packet"
---

# XDP & DPDK Kernel-Bypass Firewall Skill

The `xdp-dpdk-kernel-bypass-firewall` skill specializes in ultra-low-latency, line-rate network packet filtering and DDoS mitigation at 10, 40, 100, and 200 Gbps speeds using eXpress Data Path (XDP driver and offload modes) and the Data Plane Development Kit (DPDK).

## Core Capabilities
1. **Line-Rate eXpress Data Path (XDP)**: Synthesizes direct NIC-driver-level BPF programs that process raw Ethernet packets before Linux network stack (`sk_buff`) allocation, achieving 14.88 to 148.8 Mpps.
2. **DPDK Kernel-Bypass Poll-Mode Driver (PMD)**: Manages lockless lock-free ring buffers (`rte_ring`), Hugepages memory pools (`rte_mempool`), and SIMD-vectorized bulk packet burst reception (`rte_eth_rx_burst`).
3. **Zero-Allocation Packet Inspection**: Verifies single-cache-line packet header parsing (Ethernet, IPv4/IPv6, TCP/UDP) with strict pointer bounds checks (`data + offset <= data_end`).
4. **Autonomous DDoS Mitigation**: Deploys dynamic BPF maps (LPM trie, LRU hash) for SYN flood filtering, connection tracking rate limits, and wire-speed packet redirection (`XDP_TX`, `XDP_REDIRECT`).

## Strict Operational Invariants

- **ALWAYS**:
  - Perform strict verifier-mandated boundary checks before dereferencing any byte in the packet payload (`cursor + sizeof(struct) <= data_end`).
  - Align all packet structures and memory-mapped buffers to 64-byte cache line boundaries to avoid false sharing across core workers.
  - Return deterministic XDP actions (`XDP_DROP`, `XDP_PASS`, `XDP_TX`, `XDP_REDIRECT`) based on verified filter policies.
  - Process packets in batched bursts (e.g. 32 or 64 packets per poll) to amortize PCIe transaction latency.

- **NEVER**:
  - NEVER allocate dynamic heap memory, invoke standard libc syscalls, or trigger thread context switches in the fast-path packet loop.
  - NEVER acquire spinlocks, mutexes, or atomic CAS loops that could stall high-throughput pipeline stages.
  - NEVER permit unbounded loops in BPF/XDP kernels; every iteration limit must be statically provable to the kernel verifier.
  - NEVER access unverified memory offsets beyond the verified packet slice boundaries.

- **MANDATORY**:
  - Guarantee per-packet processing budget is strictly within hardware line-speed limits (e.g., <= 6.72 ns budget for 64-byte minimum frames at 100 Gbps).
  - Use lock-free Single Producer Single Consumer (SPSC) or Multi Producer Multi Consumer (MPMC) ring queues for zero-copy IPC between worker threads and userspace collectors.
  - Implement automatic fallback from XDP driver-mode to generic-mode if hardware NIC offload is unavailable.

- **STRICT_REJECT**:
  - Reject any packet filter implementation containing heap allocations (`malloc`, `Box::new`, `Vec::new`) in the packet processing loop.
  - Reject BPF programs that exceed the 512-byte stack frame limit or omit `data + sizeof(hdr) <= data_end` bounds assertions.
  - Reject DPDK configurations running without pre-allocated 2MB or 1GB hugepages.
