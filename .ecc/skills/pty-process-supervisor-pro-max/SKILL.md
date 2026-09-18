---
name: pty-process-supervisor-pro-max
description: Autonomous Master Engine for PTY Process Supervision, Long-Running Background Daemons, Interactive Dev Servers, Ring-Buffered Stream Capture, and Job Lifecycle Isolation. Enforces deterministic process supervision, non-blocking asynchronous I/O, bidirectional stdin piping, zero-orphan process containment, and cross-platform process tree termination across Windows, Linux, and macOS. Triggers: pty, terminal, process-supervisor, background-process, dev-server, daemon-runner, process-manager, pty-supervisor, ring-buffer, task-runner, pty-pro-max.
version: 1.0.0
tags:
  - pty
  - process-supervisor
  - background-tasks
  - dev-servers
  - ring-buffer
  - streaming
  - cross-platform
  - async-io
  - zero-zombies
compatibility: ">=0.2.0"
---

# PTY Process Supervisor Pro Max: Autonomous Background Daemon & Stream Management Engine

## Purpose & Scope
Autonomous development workflows require running long-lived background processes: development servers (e.g., Vite, Next.js, Axum, Actix), test watchers (`cargo watch`, `jest --watch`), background database containers, compilers, and interactive REPLs. Blocking an agent turn on a server loop freezes execution, while fire-and-forget backgrounding without supervision spawns orphaned zombie processes that exhaust system ports and memory.

The `pty-process-supervisor-pro-max` skill establishes Tagisan's authoritative process supervision and stream management architecture. It codifies the 6 architectural pillars governing non-blocking asynchronous process spawning, ring-buffered stdout/stderr capture, interactive stdin piping, granular process lifecycle querying, deterministic process tree termination, and cross-platform safety guarantees across Windows, Linux, and macOS.

---

## Pillar PTY-01: Shared In-Memory Process Registry & Lifecycle Management

### 1. Centralized Autonomous Process Registry
All background processes executed by agents or tools register in a globally accessible, thread-safe in-memory registry:
- **Unique Process Identifier (`id`)**: Stable reference string (e.g. `dev_server_web`, `cargo_watch_backend`, or auto-generated UUID/timestamp).
- **Process Handle & Metadata**: System PID, start timestamp, executing binary, working directory, and environment overrides.
- **Process State Machine**:
  - `Running`: Process actively executing with open stream descriptors.
  - `Exited(i32)`: Process completed naturally with recorded exit status code.
  - `Killed`: Terminated intentionally via supervisory signal.
  - `Failed(String)`: Process failed to spawn or encountered fatal OS error.

### 2. Double-Registration & Collision Prevention
Attempting to spawn an existing active process ID returns an actionable error, preventing duplicate instances from contending on identical network ports or file locks.

---

## Pillar PTY-02: Ring-Buffered Asynchronous Stream Capture

### 1. Fixed-Capacity Ring Buffering
Long-running daemons can emit gigabytes of output over hours of execution. Storing unbounded output in memory causes out-of-memory panics:
- Stdout and Stderr are ingested asynchronously line-by-line via `tokio::io::BufReader`.
- Lines are pushed into a thread-safe ring buffer (`VecDeque<String>`) with an invariant maximum capacity (e.g. 1,000 lines).
- When capacity is reached, oldest lines are evicted in $O(1)$ time, guaranteeing constant memory footprint regardless of runtime duration.

### 2. High-Performance Log Tail & Offset Pagination
The `logs` action supports:
- `tail: true` (default): Returns the most recent $N$ lines (default: 50 lines).
- `offset`: Fetches logs starting from a specific line index for deterministic incremental streaming.
- ANSI color stripping and sanitization options for clean LLM context consumption.

---

## Pillar PTY-03: Bidirectional Interactive Stdin Piping

### 1. Non-Blocking Stdin Dispatch
Interactive processes (REPLs, CLI wizards, confirmation prompts, terminal utilities) demand responsive stdin inputs without deadlocks:
- Child process stdin is opened with `Stdio::piped()`.
- An asynchronous message channel (`mpsc::Sender<String>`) queues outbound input lines.
- Background worker task writes queued bytes to stdin and immediately flushes stream descriptors.
- Supports newline auto-termination and arbitrary binary/text payloads.

---

## Pillar PTY-04: Cross-Platform Process Tree Termination (Zero Zombie Guarantee)

### 1. Hierarchical Process Tree Destruction
Modern dev tools spawn multi-tiered child process hierarchies (e.g. `npm run dev` spawns `node` which spawns `esbuild` workers). Standard `child.kill()` terminates only the top-level parent, leaving orphaned children holding network ports:
- **Windows**: Executes native process tree termination via `taskkill /F /T /PID <pid>` or Win32 Job Object closure, ensuring all descendants are purged instantly.
- **Linux & macOS**: Sends `SIGTERM` to the process group (`-PID`), allowing 500ms for graceful teardown, followed by unconditional `SIGKILL` (`kill -9 -PID`) if any descendants linger.
- **Resource Reclaiming**: Verifies that open file handles and socket bindings are completely freed before marking process as stopped.

---

## Pillar PTY-05: Real-Time Process Health & Resource Telemetry

### 1. Granular Status Inspection
The `status` action queries live operating system counters:
- Process liveness check (`try_wait()` with zero polling delay).
- Uptime duration in human-readable and millisecond precision.
- Memory usage (working set bytes) and CPU utilization where available.
- Stream status (open/closed for stdin, stdout, stderr).

---

## Pillar PTY-06: Autonomous Tool-Use Protocols

### 1. Standard Dev Server Workflow
1. **Launch**: Agent executes `pty_supervisor` with `action: "start"`, `command: "bun run dev"`, `id: "web_server"`.
2. **Readiness Probe**: Agent sleeps or waits for log readiness indicator via `action: "logs"` inspecting for `Local: http://localhost:5173`.
3. **Execution**: Agent executes integration tests or headless browser navigations against localhost.
4. **Clean Teardown**: In `finally` block, agent executes `action: "stop"`, guaranteeing system cleanliness.
