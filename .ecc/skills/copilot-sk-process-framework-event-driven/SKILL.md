---
name: copilot-sk-process-framework-event-driven
description: "Semantic Kernel Process Framework: step-based stateful workflows, routing, and event subscriptions."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Enterprise Integration Patterns: Designing, Building, and Deploying - Gregor Hohpe"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-process-framework", "step-based-workflows", "event-driven-agents", "sk-stateful-processes"]
---

# copilot-sk-process-framework-event-driven
> Based on **Enterprise Integration Patterns: Designing, Building, and Deploying - Gregor Hohpe** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Process Model: Processes comprise Steps, Events, and State, executing deterministically across distributed systems.**
2. **Event Subscriptions: Steps subscribe to specific events emitted by prior steps.**
3. **ALWAYS persist process state to allow long-running workflow resumption.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-process-framework-event-driven.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-sk-process-framework-event-driven actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design complex, long-running agent workflows using the Semantic Kernel Process Framework for resilient step orchestration.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-process-framework-event-driven.**
- **Unmonitored runtime execution without telemetry in copilot-sk-process-framework-event-driven.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-process-framework"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
