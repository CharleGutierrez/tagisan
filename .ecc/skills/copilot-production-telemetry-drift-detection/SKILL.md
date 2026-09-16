---
name: copilot-production-telemetry-drift-detection
description: "Monitoring distribution drift in production user queries, intent classification, and failure modes."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Continuous Evaluation & Monitoring for AI in CI/CD - Chip Huyen"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["telemetry-drift-detection", "production-monitoring-ai", "user-query-drift", "intent-drift-analysis"]
---

# copilot-production-telemetry-drift-detection
> Based on **Continuous Evaluation & Monitoring for AI in CI/CD - Chip Huyen** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Drift Metrics: Track semantic embedding drift of user queries over time to detect shifting user needs.**
2. **Error Distribution: Monitor changes in unhandled query rates and fallback topic triggers.**
3. **ALWAYS retrain or refine grounding prompts when distribution drift exceeds statistical thresholds.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-production-telemetry-drift-detection.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-production-telemetry-drift-detection actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Monitor production agent telemetry for semantic drift and user behavior changes, triggering proactive model updates.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-production-telemetry-drift-detection.**
- **Unmonitored runtime execution without telemetry in copilot-production-telemetry-drift-detection.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "telemetry-drift-detection"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
