---
name: ebpf-kernel-telemetry-tracer
description: Linux kernel eBPF telemetry, tracing, and observability runtime. Synthesizes Aya and libbpf safe probes (kprobes, tracepoints, uprobes, perf_events), audits BPF verifier invariants (512-byte stack bounds, loop termination, packet bounds), and diagnoses futex lock contention and CPU LLC-cache-miss bottlenecks.
version: 1.0.0
tags:
  - ebpf
  - kernel-telemetry
  - libbpf
  - aya
  - tracing
  - kprobes
  - tracepoints
  - futex-contention
  - cache-miss
  - bpf-verifier
triggers:
  - ebpf-kernel-telemetry-tracer
  - ebpf-tracer
  - kernel-tracing
  - aya-bpf
  - libbpf-rust
  - kprobe-telemetry
  - futex-contention
  - perf-event-trace
compatibility: ">=0.2.0"
---

# Linux Kernel eBPF Telemetry & Tracing Sentinel

## Purpose & Scope
Modern cloud-native and high-throughput systems experience performance regressions and latency spikes in kernel-space subsystems (futex lock contention, scheduler queues, TCP socket stalls, and L3/LLC cache thrashing). Traditional sampling profilers either incur prohibitive overhead or fail to capture nanosecond-granularity kernel state transitions.

This skill provides an autonomous engineering framework for synthesizing, verifying, and deploying high-performance eBPF (Extended Berkeley Packet Filter) telemetry programs using Aya (pure Rust eBPF) and libbpf. It enforces strict BPF verifier compliance, guarantees zero kernel crash hazard, establishes low-overhead asynchronous telemetry streaming via ring buffers, and diagnoses systemic bottlenecks.

---

## 1. Operational Invariants (eBPF Telemetry Protocol)

### Invariant 1: BPF Verifier Compliance & Stack Limit Proofs
- **MANDATORY**: Total local stack frame allocation per eBPF program must strictly not exceed 512 bytes (`MAX_BPF_STACK = 512`). All larger state structs must reside in BPF maps (e.g. `BPF_MAP_TYPE_PERCPU_ARRAY` or `BPF_MAP_TYPE_RINGBUF`).
- **ALWAYS**: Guarantee loop bounds statically. Any loop within the eBPF kernel program must use constant loop counts verified at compile-time or explicit `#pragma unroll` / `#pragma clang loop unroll(full)`.
- **MANDATORY**: Before accessing network packet buffers or memory offsets, verify pointer boundaries against `ctx.data_end` (`data + offset <= data_end`).
- **STRICT_REJECT**: Reject any eBPF program containing variable-bounded loops, unbounded recursion, or unverified stack memory accesses that fail kernel BPF verifier static analysis.

### Invariant 2: Zero Kernel Panic Hazard & Safe Pointer Dereferencing
- **MANDATORY**: Kernel-space memory reads must exclusively use verified helper functions (`bpf_probe_read_kernel`, `bpf_probe_read_user`, or Aya's safe abstractions).
- **NEVER**: Directly dereference raw kernel pointers without verifier-validated helper extraction.
- **ALWAYS**: Check return codes of eBPF helper calls and handle error states gracefully without panicking or terminating prematurely.
- **STRICT_REJECT**: Reject any probe implementation that attempts direct arithmetic or unchecked dereferencing on arbitrary kernel memory addresses.

### Invariant 3: Minimal Ringbuffer Overhead & Non-Blocking Asynchronous Exfiltration
- **MANDATORY**: Telemetry event exfiltration from kernel space to userspace must utilize `BPF_MAP_TYPE_RINGBUF` (ringbuffer) instead of obsolete `perf_event` arrays, minimizing memory allocations and memory-barrier overhead.
- **ALWAYS**: Reserve ringbuffer entries with `ring_buf.reserve()` and commit with `entry.submit()`.
- **NEVER**: Block kernel execution contexts waiting for userspace drain. If the ringbuffer is saturated, increment an atomic drop counter in a per-CPU metrics map and drop the event cleanly.
- **STRICT_REJECT**: Reject telemetry architectures that execute synchronous processing or blocking operations inside kernel hook contexts.

### Invariant 4: Futex Contention & CPU Cache-Miss Hardware Counter Diagnostics
- **MANDATORY**: When diagnosing latency anomalies, correlate syscall tracepoints (`sys_enter_futex`, `sys_exit_futex`) with thread IDs, calculating wait durations and lock contention hot-spots.
- **ALWAYS**: Profile hardware performance counters (`PERF_COUNT_HW_CACHE_MISSES`, `PERF_COUNT_HW_BRANCH_MISSES`) using `perf_event` probes to diagnose memory alignment and cacheline bouncing regressions.
- **STRICT_REJECT**: Reject telemetry analyses that report average latency without high-percentile (p99, p99.9) distributions and kernel lock contention durations.

---

## 2. Canonical eBPF Implementation Patterns

### Safe Aya Rust eBPF Kprobe with RingBuffer (Kernel Space)
```rust
#![no_std]
#![no_main]

use aya_bpf::{
    macros::{kprobe, map},
    maps::RingBuf,
    programs::ProbeContext,
    helpers::bpf_ktime_get_ns,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KernelTraceEvent {
    pub pid: u32,
    pub tgid: u32,
    pub timestamp_ns: u64,
    pub duration_ns: u64,
    pub event_id: u32,
}

#[map]
static RING_BUF: RingBuf = RingBuf::with_byte_size(1024 * 1024, 0);

#[kprobe]
pub fn trace_futex_wait(ctx: ProbeContext) -> u32 {
    match try_trace_futex_wait(&ctx) {
        Ok(ret) => ret,
        Err(_) => 1,
    }
}

#[inline(always)]
fn try_trace_futex_wait(ctx: &ProbeContext) -> Result<u32, i64> {
    let tgid_pid = ctx.pid();
    let pid = (tgid_pid & 0xFFFF_FFFF) as u32;
    let tgid = (tgid_pid >> 32) as u32;
    let ts = unsafe { bpf_ktime_get_ns() };

    // Strict verifier stack limit: keep local allocations small (< 64 bytes)
    let event = KernelTraceEvent {
        pid,
        tgid,
        timestamp_ns: ts,
        duration_ns: 0,
        event_id: 101, // Futex contention marker
    };

    if let Some(mut entry) = RING_BUF.reserve::<KernelTraceEvent>(0) {
        entry.write(event);
        entry.submit(0);
    }
    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

### Aya Userspace RingBuffer Exfiltration Loader (Rust)
```rust
use aya::maps::ring_buf::RingBuf;
use aya::programs::KProbe;
use aya::Bpf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct TelemetryCollector {
    bpf: Bpf,
    running: Arc<AtomicBool>,
}

impl TelemetryCollector {
    pub fn load_and_attach(bpf_bytes: &[u8], fn_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut bpf = Bpf::load(bpf_bytes)?;
        let program: &mut KProbe = bpf.program_mut("trace_futex_wait")
            .ok_or("Program trace_futex_wait not found")?
            .try_into()?;
        program.load()?;
        program.attach(fn_name, 0)?;

        Ok(Self {
            bpf,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn consume_telemetry(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let ring_buf = RingBuf::try_from(self.bpf.map_mut("RING_BUF").ok_or("RING_BUF not found")?)?;
        while self.running.load(Ordering::Relaxed) {
            while let Some(item) = ring_buf.next() {
                // Parse and dispatch event
                let raw_bytes: &[u8] = &item;
                println!("Exfiltrated telemetry event: {} bytes", raw_bytes.len());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(())
    }
}
```
