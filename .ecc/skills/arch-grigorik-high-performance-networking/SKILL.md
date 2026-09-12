---
name: arch-grigorik-high-performance-networking
description: "Modern transport protocols: HTTP/2 multiplexing, HPACK header compression, HTTP/3 (QUIC over UDP), TLS 1.3 0-RTT handshakes, and transport latency optimization."
triggers: ["grigorik-networking", "http2-multiplexing", "quic-http3", "tls-optimization", "head-of-line-blocking", "hpack", "transport-performance"]
---

# arch-grigorik-high-performance-networking
> Based on **High Performance Browser Networking - Ilya Grigorik**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Internal microservice transport must utilize HTTP/2 or HTTP/3 multiplexing to stream multiple concurrent requests over a single TCP/UDP connection.**
2. **ALWAYS: Enable TLS 1.3 session resumption (0-RTT / session tickets) to eliminate extra round-trip handshakes.**
3. **NEVER: Open multiple parallel TCP connections to the same host when HTTP/2 multiplexing is available.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Optimize transport layers with HTTP/2 multiplexing, QUIC packet loss isolation, and TLS 1.3 session resumption. Eliminate head-of-line blocking.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Opening 6 parallel HTTP/1.1 connections to bypass head-of-line blocking in modern environments.**
- **Disabling connection reuse, forcing TLS handshakes on every payload.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-grigorik-high-performance-networking"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
