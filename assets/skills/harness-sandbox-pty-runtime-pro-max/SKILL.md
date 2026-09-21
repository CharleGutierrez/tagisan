---
name: harness-sandbox-pty-runtime-pro-max
description: Advanced Sandboxing & PTY Interactive Runtimes for Tagisan. Cross-platform isolation (Linux Landlock v1-v3 + Seccomp BPF, macOS Seatbelt sandbox-exec, Windows AppContainer/Job Object), interactive PTY/pseudoterminal emulation (controlling vim, fzf, htop, curses TUIs), ephemeral micro-containers/jails, and VCR HTTP/TLS fixture recording & replay. Enforces no root escape, deterministic input-output capture, zero hanging on blocked child processes, and a 1MB memory capture ceiling.
version: 1.0.0
tags:
  - sandboxing
  - pty-runtime
  - landlock
  - seccomp
  - macos-seatbelt
  - windows-appcontainer
  - pseudoterminal
  - tui-automation
  - vcr-replay
  - security-isolation
triggers:
  - sandbox-pty-runtime
  - landlock-sandbox
  - seccomp-filter
  - seatbelt-profile
  - appcontainer-jail
  - pty-session
  - tui-controller
  - vim-fzf-automation
  - vcr-recording
  - network-replay
compatibility: ">=0.2.0"
---

# Advanced Sandboxing & PTY Interactive Runtimes: Secure, Deterministic Execution & TUI Control

## Purpose & Architectural Mandate
Executing untrusted code, running third-party binaries, and controlling interactive Terminal User Interfaces (TUIs) like `vim`, `fzf`, and `htop` pose severe operational hazards:
1. **Security Vulnerabilities**: Malicious or buggy binaries may exfiltrate secrets, tamper with host files, or launch fork-bombs.
2. **Interactive Deadlocks**: Programs expecting a TTY hang indefinitely when spawned over standard non-interactive pipes.
3. **Non-Deterministic Network Dependency**: Tests and agent evaluations break when external APIs change or network connectivity drops.

The `harness-sandbox-pty-runtime-pro-max` skill provides comprehensive, cross-platform process isolation (Landlock, Seccomp, Seatbelt, AppContainer), full pseudoterminal (PTY) emulation for TUI manipulation, and VCR HTTP fixture recording and replay.

---

## 1. Operational Invariants

```
+---------------------------------------------------------------------------------------------------+
|                                SANDBOX & PTY RUNTIME INVARIANTS                                   |
+---------------------------------------------------------------------------------------------------+
|  1. NO ROOT ESCAPE & PRIVILEGE ELEVATION                                                          |
|     `PR_SET_NO_NEW_PRIVS` must be applied before child exec. All capabilities dropped. User      |
|     namespaces mapped to non-root UID/GID. Sandboxed process can NEVER acquire root or sudo.      |
+---------------------------------------------------------------------------------------------------+
|  2. DETERMINISTIC INPUT-OUTPUT CAPTURE & SCREEN RECONSTRUCTION                                   |
|     All PTY byte streams must be captured and parsed through a virtual VT100/Xterm terminal       |
|     screen buffer. Cursor positions, colors, and alternate screens must be deterministically      |
|     rendered to text snapshots.                                                                   |
+---------------------------------------------------------------------------------------------------+
|  3. ZERO HANGING ON BLOCKED CHILD PROCESSES                                                       |
|     Child execution MUST be monitored via non-blocking epoll/kqueue with strict timeouts.          |
|     Escalation ladder: SIGINT -> (grace 500ms) -> SIGTERM -> (grace 500ms) -> SIGKILL.            |
+---------------------------------------------------------------------------------------------------+
|  4. 1MB MEMORY CAPTURE CEILING                                                                    |
|     PTY output buffers must utilize a circular ring buffer capped at 1MB per stream to prevent    |
|     memory exhaustion from runaway infinite loops or binary dumps.                                |
+---------------------------------------------------------------------------------------------------+
|  5. HERMETIC VCR CASSETTE HYGIENE                                                                 |
|     All network replays must strip authorization headers, cookies, and tokens (`Bearer ***`).     |
|     Replays must produce bit-for-bit identical responses without opening external sockets.        |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Cross-Platform Sandboxing Architecture

```
+----------------------------------------------------------------------------------+
|                             TAGISAN SANDBOX RUNTIME                              |
+----------------------------------------------------------------------------------+
            |                                |                           |
    (Linux Kernel)                     (macOS Darwin)            (Windows NT)
            |                                |                           |
+------------------------+      +------------------------+  +--------------------+
|  Landlock v1-v3        |      |  Seatbelt Profile      |  |  AppContainer SID  |
|  - Read-only rootfs    |      |  - sandbox-exec        |  |  - Job Object      |
|  - Write-only /tmp     |      |  - Deny file-write*    |  |  - No UI Access    |
|  - Deny network listen |      |  - Deny network-out*   |  |  - Low Integrity   |
+------------------------+      +------------------------+  +--------------------+
            |
+------------------------+
|  Seccomp BPF           |
|  - Block ptrace, reboot|
|  - Block raw sockets   |
+------------------------+
```

### 1. Linux Landlock v1-v3 + Seccomp BPF Ruleset
Landlock restricts filesystem access at the kernel level without requiring root privileges:
```rust
//! Landlock sandboxing in Rust
use std::fs::File;
use std::os::unix::io::AsRawFd;

pub struct LandlockSandbox {
    allowed_reads: Vec<String>,
    allowed_writes: Vec<String>,
}

impl LandlockSandbox {
    pub fn new() -> Self {
        Self {
            allowed_reads: vec!["/usr".into(), "/lib".into(), "/lib64".into(), "/etc".into()],
            allowed_writes: vec!["/tmp".into()],
        }
    }

    pub fn apply(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Enforce PR_SET_NO_NEW_PRIVS first
        unsafe {
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                return Err("Failed to set PR_SET_NO_NEW_PRIVS".into());
            }
        }
        // Apply Landlock ruleset via landlock syscalls
        // (Ruleset created with LANDLOCK_ACCESS_FS_READ_FILE | EXECUTE, etc.)
        Ok(())
    }
}
```

### 2. macOS Seatbelt (`sandbox-exec` / `.sb` Profile)
```scheme
;; Tagisan Hermetic Seatbelt Profile (macOS)
(version 1)
(deny default)

;; Allow process execution and library reading
(allow process-exec (subpath "/bin") (subpath "/usr/bin") (subpath "/usr/local/bin"))
(allow file-read* (subpath "/usr") (subpath "/System") (subpath "/Library") (subpath "/dev"))

;; Allow restricted write only in designated scratch directory
(allow file-write* (subpath "/private/tmp/tagisan_jail"))

;; Deny outbound sockets unless explicitly permitted
(deny network*)
```

### 3. Windows AppContainer & Job Object Isolation
- Creates a dedicated AppContainer security profile with low integrity SID.
- Applies a `JOB_OBJECT_LIMIT_PROCESS_MEMORY` ceiling (e.g. 512MB).
- Disallows clipboard access, window handle creation, and network egress.

---

## 3. Interactive PTY Emulation & TUI Control

When interacting with full-screen terminal programs (`vim`, `fzf`, `htop`, `nano`), standard stdin/stdout pipes fail because the program checks `isatty(STDIN_FILENO)`. Tagisan allocates a real pseudoterminal master/slave pair.

```rust
//! PTY Master Controller with Virtual Terminal Screen Buffer
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::RawFd;

pub struct PtyController {
    master_fd: RawFd,
    child_pid: libc::pid_t,
    screen_cols: u16,
    screen_rows: u16,
}

impl PtyController {
    pub fn spawn(cmd: &str, args: &[&str], cols: u16, rows: u16) -> Result<Self, std::io::Error> {
        let mut master_fd: RawFd = 0;
        let mut slave_fd: RawFd = 0;

        unsafe {
            let mut win = libc::winsize {
                ws_row: rows,
                ws_col: cols,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };

            let pid = libc::forkpty(&mut master_fd, std::ptr::null_mut(), std::ptr::null_mut(), &mut win);
            if pid < 0 {
                return Err(std::io::Error::last_os_error());
            } else if pid == 0 {
                // Child: execute command
                let c_cmd = std::ffi::CString::new(cmd).unwrap();
                let mut c_args = vec![c_cmd.clone()];
                for arg in args {
                    c_args.push(std::ffi::CString::new(*arg).unwrap());
                }
                let mut c_ptrs: Vec<*const libc::c_char> = c_args.iter().map(|s| s.as_ptr()).collect();
                c_ptrs.push(std::ptr::null());

                libc::execvp(c_cmd.as_ptr(), c_ptrs.as_ptr());
                libc::_exit(127);
            }

            Ok(Self {
                master_fd,
                child_pid: pid,
                screen_cols: cols,
                screen_rows: rows,
            })
        }
    }

    pub fn send_input(&mut self, data: &[u8]) -> std::io::Result<()> {
        let n = unsafe { libc::write(self.master_fd, data.as_ptr() as *const libc::c_void, data.len()) };
        if n < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn send_keys(&mut self, keys: &str) -> std::io::Result<()> {
        self.send_input(keys.as_bytes())
    }
}
```

---

## 4. VCR HTTP/TLS Fixture Recording & Replay

```yaml
# Tagisan VCR Cassette: vcr_fixtures/github_api_repo.yaml
version: 1.0.0
recorded_at: 2026-09-21T12:00:00Z
http_interactions:
  - request:
      method: GET
      uri: https://api.github.com/repos/tagisan/tgs
      headers:
        Accept: application/vnd.github.v3+json
        User-Agent: Tagisan-VCR/1.0
        Authorization: "[FILTERED]"
      body: ""
    response:
      status: 200
      headers:
        Content-Type: application/json; charset=utf-8
        ETag: '"w/34a9b2c8"'
      body: '{"id": 89124,"name": "tagisan","full_name": "tagisan/tgs","private": false}'
```

### Deterministic Replay Mode
When `TAGISAN_VCR_MODE=replay` is enabled:
- All outbound network calls are intercepted via an in-memory loopback proxy or DNS override (`127.0.0.1:port`).
- Matching requests return the recorded response with microsecond latency.
- Unmatched requests immediately fail with `ERR_VCR_UNRECORDED_INTERACTION` instead of attempting real network egress.
