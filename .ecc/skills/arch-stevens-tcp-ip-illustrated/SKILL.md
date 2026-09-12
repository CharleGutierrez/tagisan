---
name: arch-stevens-tcp-ip-illustrated
description: "Low-level networking foundations: TCP 3-way handshake, flow control, sliding window, congestion control (CUBIC/BBR), TIME_WAIT states, and socket options (SO_REUSEADDR)."
triggers: ["stevens-tcp-ip", "tcp-flow-control", "sliding-window", "congestion-control", "time-wait", "socket-options", "tcp-handshake", "network-protocols"]
---

# arch-stevens-tcp-ip-illustrated
> Based on **TCP/IP Illustrated, Volume 1: The Protocols - W. Richard Stevens & Kevin R. Fall**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: High-volume network services must reuse persistent pooled connections (HTTP Keep-Alive / TCP connection pooling) to prevent ephemeral port exhaustion (TIME_WAIT).**
2. **ALWAYS: Configure TCP_NODELAY (disable Nagle's algorithm) for latency-sensitive interactive RPC payloads to eliminate 40ms delayed ACK stalls.**
3. **NEVER: Create new TCP sockets per request inside hot request loops.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Configure socket options (TCP_NODELAY, SO_REUSEADDR). Tune TCP receive/send buffer windows. Implement persistent connection pooling to avoid TIME_WAIT socket storms.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Opening and closing TCP connections for every single API call.**
- **Suffering 40ms latency spikes due to Nagle's algorithm interacting with delayed ACKs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-stevens-tcp-ip-illustrated"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
