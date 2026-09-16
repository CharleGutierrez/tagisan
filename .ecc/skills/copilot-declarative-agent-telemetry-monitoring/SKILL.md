---
name: copilot-declarative-agent-telemetry-monitoring
description: "App usage analytics, diagnostic event logging, and audit logs in Purview and Microsoft Entra."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Deploying and Managing Microsoft 365 Copilot - J. Peter Bruzzese"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["agent-telemetry", "usage-analytics", "purview-audit-logs", "copilot-monitoring"]
---

# copilot-declarative-agent-telemetry-monitoring
> Based on **Deploying and Managing Microsoft 365 Copilot - J. Peter Bruzzese** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Audit Integration: All agent invocations, tool actions, and data accesses recorded in unified M365 Audit Log.**
2. **Usage Telemetry: Teams Admin Center metrics tracking active users, session frequency, and execution errors.**
3. **MANDATORY monitoring of rate limiting and 4xx/5xx HTTP errors on backend action endpoints.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-declarative-agent-telemetry-monitoring.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-declarative-agent-telemetry-monitoring.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement continuous monitoring of agent action endpoints using Application Insights. Set up alerting for latency degradation and anomalous error rates.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Deploying agents without telemetry hooks, operating blind to real-world user failures.**
- **Ignoring spike alerts in M365 Copilot admin dashboard indicating broken plugin endpoints.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "agent-telemetry"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
