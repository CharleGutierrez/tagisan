---
name: copilot-hardware-npu-directml-telemetry
description: "Windows Copilot+ PC on-device NPU DirectML execution, 40+ TOPS accelerator telemetry, and carbon efficiency metrics."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Windows Copilot+ Architecture & DirectML Engineering - Microsoft Hardware Systems"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-hardware-npu", "directml-execution", "copilot-plus-pc", "npu-telemetry"]
---

# copilot-hardware-npu-directml-telemetry
> Based on **Windows Copilot+ Architecture & DirectML Engineering - Microsoft Hardware Systems** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Hardware Acceleration: DirectML execution layer abstracting Qualcomm Snapdragon X, Intel Lunar Lake, and AMD Strix Point NPUs.**
2. **40+ TOPS Invariant: High-throughput on-device SLM inference (Phi-Silica, Whisper) offloaded from CPU/GPU.**
3. **MANDATORY power and thermal throttling monitoring during sustained local inference loops.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-hardware-npu-directml-telemetry.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-hardware-npu-directml-telemetry.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Architect hybrid applications that route low-latency, privacy-critical reasoning to local NPU via DirectML and heavy reasoning to Azure OpenAI.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Blocking UI threads with synchronous local model tensor operations.**
- **Failing to implement automatic fallback to cloud LLM when NPU hardware is unavailable.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-hardware-npu"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
