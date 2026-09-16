---
name: copilot-sk-enterprise-observability
description: "OpenTelemetry instrumentation, trace spans, token usage telemetry, and Application Insights integration."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Observability Engineering: Achieving Operational Excellence - Charity Majors"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-observability", "opentelemetry-sk", "token-usage-telemetry", "application-insights-sk"]
---

# copilot-sk-enterprise-observability
> Based on **Observability Engineering: Achieving Operational Excellence - Charity Majors** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **OTel Instrumentation: Semantic Kernel emits standard `Microsoft.SemanticKernel` Activity spans.**
2. **Metrics Tracking: Monitor `semantic_kernel.tokens.prompt`, `semantic_kernel.tokens.completion`, `semantic_kernel.function.duration`.**
3. **MANDATORY redacting of PII and user prompt text from production telemetry traces.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-sk-enterprise-observability.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-enterprise-observability.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Instrument Semantic Kernel applications with OpenTelemetry exporters to gain granular visibility into model latency, cost, and tool health.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-enterprise-observability.**
- **Unmonitored runtime execution without telemetry in copilot-sk-enterprise-observability.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-observability"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
