---
name: posix-shell-sanitizer
description: POSIX shell AST validation, command injection prevention, and destructive execution interceptors
---

# POSIX Shell Sanitizer & Command Interception

## Safety Rules
1. Prohibited Commands: Block destructive operations (`rm -rf /`, `mkfs`, `dd if=/dev/zero`, `:(){ :|:& };:`).
2. Argument Whitelisting: Enforce strict parameter validation on shell commands.
3. Path Confinement: Reject commands with traversal escapes (`../../../`) targeting root filesystems.
4. Subshell Isolation: Execute commands in dedicated subprocesses with strict timeouts and resource limits.
