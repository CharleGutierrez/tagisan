---
name: copilot-end-to-end-enterprise-action-choreography
description: "Orchestrating complex distributed transactions across SAP, Salesforce, and ServiceNow via Copilot."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Enterprise Integration Patterns: Designing, Building, and Deploying - Gregor Hohpe"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["enterprise-action-choreography", "distributed-saga-copilot", "cross-system-transactions", "enterprise-workflows"]
---

# copilot-end-to-end-enterprise-action-choreography
> Based on **Enterprise Integration Patterns: Designing, Building, and Deploying - Gregor Hohpe** (Cluster 15)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Saga Choreography: Coordinate multi-system business actions via forward actions and compensating rollback actions.**
2. **Idempotency Invariant: Every distributed step MUST support idempotent execution via unique idempotency keys.**
3. **MANDATORY complete audit trail recording every state change and compensating transaction.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-end-to-end-enterprise-action-choreography.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-end-to-end-enterprise-action-choreography.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Orchestrate complex, multi-system enterprise workflows across SAP, Salesforce, and ServiceNow with transactional integrity.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-end-to-end-enterprise-action-choreography.**
- **Unmonitored runtime execution without telemetry in copilot-end-to-end-enterprise-action-choreography.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "enterprise-action-choreography"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
