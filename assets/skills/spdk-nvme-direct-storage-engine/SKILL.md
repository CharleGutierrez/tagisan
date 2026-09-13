---
name: spdk-nvme-direct-storage-engine
description: "Asynchronous zero-copy user-space NVMe driver, Storage Performance Development Kit (SPDK), hugepages memory DMA, and lockless submission/completion queue engine."
version: 0.2.0
tags:
  - storage
  - spdk
  - nvme
  - pcie
  - dma
  - zero-copy
  - iops
  - polled-mode
compatibility: ">=0.2.0"
triggers:
  - "spdk"
  - "nvme"
  - "polled mode"
  - "submission queue"
  - "completion queue"
  - "hugepages"
  - "dma"
  - "direct io"
  - "iops"
  - "pcie gen4"
  - "pcie gen5"
---

# SPDK NVMe Direct Storage Engine Skill

The `spdk-nvme-direct-storage-engine` skill provides ultra-high-throughput, sub-microsecond latency direct user-space NVMe block storage management leveraging the Storage Performance Development Kit (SPDK), bypassing kernel VFS and block layers via polled-mode drivers (PMD) and zero-copy Direct Memory Access (DMA).

## Core Capabilities
1. **Kernel-Bypass Polled-Mode Driver**: Eliminates kernel interrupt handling overhead, context switches, and syscall costs by continuously polling NVMe Completion Queues (CQ).
2. **Lockless Asynchronous Queue Pairs**: Allocates dedicated Submission Queue (SQ) and Completion Queue (CQ) pairs per CPU core, achieving multi-million IOPS per controller without cross-thread locking.
3. **Zero-Copy DMA Buffer Management**: Allocates 2MB/1GB hugepage memory pools mapped directly to PCIe Physical Memory Attributes (PMA), avoiding intermediate buffer copies.
4. **Hardware Bus Saturation Profiling**: Optimizes queue depths (32, 64, 128, 256) to fully saturate PCIe Gen4 (up to 8 GB/s x4) and Gen5 (up to 16 GB/s x4) lane bandwidth.

## Strict Operational Invariants

- **ALWAYS**:
  - Align all DMA data buffers to physical 4KB memory page boundaries (`spdk_dma_zmalloc` with 4096-byte alignment).
  - Assign dedicated NVMe queue pairs to individual pinning threads to prevent synchronization bottlenecks.
  - Check return status on NVMe command submissions and handle queue-full backpressure via bounded ring buffers.
  - Pin polling worker threads to specific NUMA nodes matching the target NVMe PCIe controller.

- **NEVER**:
  - NEVER execute blocking POSIX I/O syscalls (`read`, `write`, `fsync`, `ioctl`) in the polled-mode event thread.
  - NEVER use standard heap allocations (`malloc`, `free`) for DMA transfer targets; use pinned hugepage memory only.
  - NEVER share NVMe submission queues across multiple OS threads without atomic locking protocols (prefer per-thread queues).
  - NEVER free DMA buffers prior to receiving explicit completion notifications on the associated Completion Queue.

- **MANDATORY**:
  - Verify hugepages configuration (`/dev/hugepages` or `/dev/hugepages1G`) and VFIO / UIO kernel modules are loaded before initializing user-space NVMe drivers.
  - Provide asynchronous callback mechanics or future completion tokens for every dispatched block write/read command.
  - Implement write-barrier and flush guarantees when committing transactional log segments.

- **STRICT_REJECT**:
  - Reject storage I/O workflows that perform page unaligned or non-DMA memory transfers to NVMe devices.
  - Reject configurations that run multi-threaded access against a single non-thread-safe NVMe queue pair.
  - Reject code that invokes blocking sleeps or locks inside the polled-mode event loop.
