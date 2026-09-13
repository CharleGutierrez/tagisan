---
name: rr-time-travel-deterministic-debugger
description: "rr & Linux perf_event deterministic record/replay debugging, reverse execution, reverse watchpoints, and memory corruption bisection."
version: 0.2.0
tags:
  - rr
  - debugging
  - time-travel
  - reverse-execution
  - perf-event
  - memory-corruption
  - heisenbug
  - gdb
  - determinism
compatibility: ">=0.2.0"
triggers:
  - "rr"
  - "time-travel debugging"
  - "reverse execution"
  - "reverse watchpoint"
  - "perf_event"
  - "heisenbug"
  - "record replay"
  - "gdb reverse"
  - "memory corruption bisection"
---

# rr Time-Travel Deterministic Debugger Skill

The `rr-time-travel-deterministic-debugger` skill provides deterministic record-and-replay debugging for Linux applications, harnessing CPU performance monitoring counters (PMU `perf_event`), reverse execution, memory watchpoints, and algorithmic timeline bisection to eliminate elusive Heisenbugs and memory corruption bugs.

## Core Capabilities

1. **Deterministic Recording Environment Planning**: Configures single-core CPU pinning (`taskset -c 0`), ASLR deactivation (`setarch -R`), and Linux PMU performance counters (`kernel.perf_event_paranoid <= 1`) to guarantee bit-accurate repeatable recording.
2. **Reverse Execution Automation**: Synthesizes non-interactive GDB script automation executing `reverse-continue`, `reverse-step`, `reverse-finish`, and hardware watchpoints backwards in time.
3. **Algorithmic Heisenbug Bisection**: Computes exact logarithmic bisection checkpoints across execution event ticks to isolate the precise instruction responsible for silent memory corruption.
4. **Non-Deterministic Trap Isolation**: Pinpoints asynchronous signal deliveries, context switches, and system call side effects recorded in the trace.

## Strict Operational Invariants

- **ALWAYS**:
  - Pin recording processes to a single physical core (`taskset -c 0`) and disable ASLR (`setarch $(uname -m) -R`) to guarantee cycle-accurate deterministic replay.
  - Verify that Linux PMU hardware performance counters are accessible (`/proc/sys/kernel/perf_event_paranoid <= 1`).
  - Use reverse execution watchpoints (`watch -l *addr` followed by `reverse-continue`) to track memory corruption backwards to the exact originating instruction.
  - Record the exact event tick index of memory corruptions or fault traps to enable algorithmic bisection.

- **NEVER**:
  - NEVER run multi-threaded recording sessions across floating CPU cores; always bind thread scheduling deterministically.
  - NEVER rely on forward single-stepping when hunting intermittent Heisenbugs; reverse-step from crash point directly to corruption origin.
  - NEVER mutate the recording trace directory (`~/.local/share/rr/`) during replay analysis.
  - NEVER ignore asynchronous signal deliveries or spurious ERESTARTSYS returns captured in the trace.

- **MANDATORY**:
  - MANDATORY configure CPU frequency governors to `performance` mode to eliminate clock jitter during recording.
  - MANDATORY compute logarithmic bisection steps ceil(log2(T_end - T_start)) when isolating data race mutations across long event timelines.
  - MANDATORY script GDB batch workflows with `set pagination off` and non-interactive logging to automate post-mortem triage.

- **STRICT_REJECT**:
  - STRICT_REJECT recording attempts where `perf_event_paranoid` is greater than 1 or PMU counters are disabled by hypervisor.
  - STRICT_REJECT replay sessions attempting forward-only debugging on non-deterministic shared memory multi-process races without thread binding.
  - STRICT_REJECT test harnesses that fail to record the exit event tick number of target binaries.
