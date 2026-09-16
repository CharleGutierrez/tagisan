---
name: copilot-latency-token-throughput-benchmarking
description: "Benchmarking Time-To-First-Token (TTFT), tokens-per-second, and roundtrip latency across models."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Observability Engineering: Achieving Operational Excellence - Charity Majors"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["latency-benchmarking", "time-to-first-token", "token-throughput", "model-performance-metrics"]
---

# copilot-latency-token-throughput-benchmarking
> Based on **Observability Engineering: Achieving Operational Excellence - Charity Majors** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Latency Metrics: Measure TTFT (aim <800ms), Tokens Per Second (TPS), and total roundtrip turn latency.**
2. **Percentile Tracking: Monitor P50, P95, and P99 latency percentiles to detect tail performance degradations.**
3. **MANDATORY automated performance regression alerts in CI.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-latency-token-throughput-benchmarking.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-latency-token-throughput-benchmarking.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Benchmark and monitor inference latency and token throughput across models and regions to guarantee responsive agent UX.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-latency-token-throughput-benchmarking.**
- **Unmonitored runtime execution without telemetry in copilot-latency-token-throughput-benchmarking.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "latency-benchmarking"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
