---
name: computer-control-os-automation-pro-max
description: Autonomous Master Engine for Operating System Automation, Native Win32/X11/macOS Computer Control, Screen Perception, Multimodal GUI Grounding, Process Supervision, and Job Isolation. Enforces deterministic UI tree introspection, SendInput/xdotool event synthesis, Windows Job Objects / cgroups v2 resource governance, headless fallback resilience, shell sanitization, and circuit-breaker protected execution. Triggers: computer-control, computer-use, os-automation, desktop-automation, rpa-automation, gui-automation, win32-automation, screen-control, mouse-keyboard-control, computer-pro-max, os-pro-max.
version: 1.0.0
tags:
  - computer-control
  - os-automation
  - rpa
  - screen-perception
  - win32
  - x11
  - sendinput
  - process-supervision
  - cgroups
  - self-healing
compatibility: ">=0.2.0"
---

# Computer Control & OS Automation Pro Max: Autonomous Sovereign Desktop & Process Control Engine

## Purpose & Scope
Autonomous agents controlling host operating systems must operate with **1000x reliability, zero hallucination, strict process containment, and deterministic safety invariants**. Speculative mouse clicking or unconstrained command execution results in corrupted system state, desktop lockups, and untracked runaway processes.

The `computer-control-os-automation-pro-max` skill establishes Tagisan's authoritative operating system automation architecture across Windows (Win32/PowerShell/GDI/Job Objects), Linux (X11/Wayland/cgroups v2/evdev), and macOS (Quartz/CoreGraphics/launchd). It codifies the 6 architectural pillars governing screen perception, input synthesis, process supervision, headless resilience, shell sanitization, and autonomous circuit breakers.

---

## Pillar CC-01: Multimodal Screen Perception, OCR & UI Tree Introspection

### 1. Multi-Monitor Coordinate Spaces & DPI Awareness
Display geometry is multi-dimensional and non-uniform:
- Virtual desktop coordinates span from negative bounds (secondary monitors positioned left/above the primary display) to large positive offsets.
- High-DPI scaling (125%, 150%, 200%) decouples physical pixel coordinates from logical points. Win32 `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` or equivalent normalization must precede coordinate emission.
- Resolution queries (`get_screen_size`) determine primary monitor bounds $(W, H)$ and total virtual screen bounds $(X_{\min}, Y_{\min}, W_{\text{virt}}, H_{\text{virt}})$.

### 2. High-Performance Screen Capture Pipeline
- **Windows**: GDI `CopyFromScreen` via `System.Drawing.Bitmap` and Win32 `BitBlt` against desktop device context `GetDC(NULL)`. Catch headless session 0 (`The handle is invalid`) and fall back to synthetic diagnostic raster buffers.
- **Linux**: Wayland compositors queried via `grim` pipe or `pipewire` desktop portals; X11 servers captured via `import -window root` or Xlib `XGetImage`.
- **macOS**: `screencapture -x` headless Quartz capture with display ID routing.
- **Headless Fallback Invariant**: In virtualized or CI environments lacking an interactive desktop compositor, screen capture operations must never panic or fail. They must generate valid standard PNG raster images (black/gray diagnostic canvas or minimal standard PNG payload) and return structured metadata with dimension and fallback indicators.

### 3. Visual Element Grounding & Accessibility Introspection
- Coordinate calculation must be bounded within $[0, W-1]$ and $[0, H-1]$.
- Accessibility tree queries (Windows UI Automation `IUIAutomation`, Linux AT-SPI, macOS `AXUIElement`) provide stable element bounding boxes $[x, y, w, h]$ and control patterns (Invoke, Value, Selection).
- Prior to clicking, visual anchors are verified via sub-image template matching or OCR character alignment, preventing misplaced clicks on shifting modals or banner notifications.

---

## Pillar CC-02: Deterministic Mouse & Keyboard Event Synthesis

### 1. Native Low-Level Event Injection
- **Windows**: Low-level hardware event synthesis via `user32.dll` `SendInput` and `mouse_event`.
  - Left button: `MOUSEEVENTF_LEFTDOWN (0x0002)` $\rightarrow$ `MOUSEEVENTF_LEFTUP (0x0004)`.
  - Right button: `MOUSEEVENTF_RIGHTDOWN (0x0008)` $\rightarrow$ `MOUSEEVENTF_RIGHTUP (0x0010)`.
  - Middle button: `MOUSEEVENTF_MIDDLEDOWN (0x0020)` $\rightarrow$ `MOUSEEVENTF_MIDDLEUP (0x0040)`.
  - Double click: Two consecutive click sequences separated by a 40–80 ms micro-delay.
- **Linux**: `xdotool` or direct Linux kernel `/dev/uinput` evdev event injection.
- **macOS**: `CGEventCreateMouseEvent` and `CGEventPost` to `kCGHIDEventTap`.

### 2. Smooth Cursor Trajectory & Drag-and-Drop Mechanics
- Direct teleportation of cursor coordinates can trigger race conditions in hovering tooltips and drop targets.
- Drag-and-drop operations (`mouse_drag`) must enforce:
  1. Position cursor at $(x_{\text{start}}, y_{\text{start}})$.
  2. Emit mouse down event.
  3. Perform linear or cubic bezier easing interpolation with intermediate positioning updates ($50\text{ ms}$ interval).
  4. Position cursor at $(x_{\text{end}}, y_{\text{end}})$.
  5. Emit mouse up event.

### 3. Keyboard Typing & Virtual-Key Chord Synthesis
- **Text Typing (`keyboard_type`)**: Unescaped keystrokes can corrupt system commands. Character payloads must escape control characters (`+`, `^`, `%`, `~`, `(`, `)`, `{`, `}`, `[`, `]`).
- **Special Key Combinations (`keyboard_press`)**: Canonical mapping for modifier keys and system shortcuts:
  - `Enter` $\rightarrow$ `{ENTER}`, `Escape` $\rightarrow$ `{ESC}`, `Tab` $\rightarrow$ `{TAB}`.
  - `Ctrl+C` $\rightarrow$ `^c`, `Ctrl+V` $\rightarrow$ `^v`, `Ctrl+A` $\rightarrow$ `^a`, `Ctrl+Z` $\rightarrow$ `^z`.
  - `Alt+F4` $\rightarrow$ `%{F4}`, `Alt+Tab` $\rightarrow$ `%{TAB}`.

---

## Pillar CC-03: Process Supervision, Job Objects & cgroup v2 Isolation

### 1. Process Tree Lifecycle & Kernel Isolation
- Every spawned child process must be bound to a hierarchical supervision container:
  - **Windows**: Windows Job Objects configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` via `SetInformationJobObject`. When the parent agent runtime terminates or aborts, all transitive child processes are atomically eradicated by the kernel, preventing orphaned background zombies.
  - **Linux**: Linux cgroups v2 (`/sys/fs/cgroup/tgs/<job_id>`) with `cgroup.kill` and `cgroup.procs` tracking.
  - **macOS**: Process groups with `setpgid` and `kill(-pgid, SIGKILL)` termination propagation.

### 2. Telemetry, Resource Limits & Process Listing (`process_list`)
- Process sampling queries OS tables (`Get-Process`, `ps -eo pid,comm,rss,pcpu`) to retrieve:
  - Unique Process Identifier (`pid`).
  - Canonical image name (`name`).
  - Physical RAM consumption in megabytes (`memory_mb`).
- Process termination (`process_kill`) enforces strict safety guardrails:
  - Prohibits termination of PID `0` (Idle), PID `4` (System Kernel), and the agent runtime's own PID (`std::process::id()`).
  - Prohibits killing critical OS system services (`csrss`, `wininit`, `services`, `lsass`, `smss`).

---

## Pillar CC-04: Headless Browser & DOM Driver Automation

### 1. Synthetic Display Buffers & CDP Orchestration
- In headless deployments (cloud VMs, containerized workflows), GUI automation bridges to Chrome DevTools Protocol (CDP) and WebDriver BiDi endpoints.
- Virtual display servers (`Xvfb :99 -screen 0 1920x1080x24` or Windows remote desktop dummy display drivers) allocate backing buffers to allow full GDI/DirectX context allocation.

### 2. DOM-to-Screen Spatial Mapping
- Elements in web applications are mapped between CSS layout coordinates, viewport scroll offsets, and physical monitor pixels:
  $$x_{\text{screen}} = x_{\text{window\_offset}} + (x_{\text{dom}} - x_{\text{scroll}}) \times \text{DPI\_scale}$$
- Guarantees exact hit-testing even when elements are embedded in nested iframes or shadow DOM trees.

---

## Pillar CC-05: POSIX & Win32 Shell Sanitization and Injection Defense

### 1. Strict Argument Vector Passing
- Never execute raw concatenated strings through `cmd.exe /C` or `sh -c` without strict tokenization.
- Utilize structured argument arrays `tokio::process::Command::args(&[...])` to bypass shell parsing bugs and command injection vectors (`&`, `|`, `;`, `` ` ``, `$()`, `%PATH%`).

### 2. Sandbox Confinement & Path Validation
- File operations (e.g. screenshot destination paths) must validate directory confinement against the agent's allocated workspace or OS temp directory, blocking path traversal (`../`) attacks.

---

## Pillar CC-06: Autonomous Self-Healing, Circuit Breakers & Human-in-the-Loop Interception

### 1. UI Synchronization & Verification Loops
- Actions must follow an **Observe $\rightarrow$ Act $\rightarrow$ Verify** loop:
  1. Capture state / locate target element.
  2. Execute input action.
  3. Re-sample active window title (`get_active_window`) or target pixel region to verify expected state transition.
  4. If transition fails, execute exponential backoff retry up to 3 times before triggering fault escalation.

### 2. Circuit Breakers & Runaway Protection
- Keystroke rate limit: Max 50 keystrokes/second to prevent buffer overflow in target message queues.
- Click runaway sentinel: Tripping the circuit breaker if more than 5 consecutive clicks occur at identical coordinates with zero UI state change.
- Global emergency halt: Unconditional abort mechanism terminating simulated inputs and freeing locked cursor/keyboard states.
