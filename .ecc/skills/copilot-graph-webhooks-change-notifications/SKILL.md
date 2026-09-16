---
name: copilot-graph-webhooks-change-notifications
description: "Creating webhook subscriptions, handling validation token handshakes, and renewal loops."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Real-Time Microsoft Graph Webhooks and Delta Queries - Glenn Turner"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["graph-webhooks", "change-notifications", "webhook-validation-token", "subscription-renewal"]
---

# copilot-graph-webhooks-change-notifications
> Based on **Real-Time Microsoft Graph Webhooks and Delta Queries - Glenn Turner** (Cluster 4)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Validation Handshake: Return validationToken query parameter within 10 seconds as plain text.**
2. **Subscription Expiration: Graph subscriptions expire; maintain background worker to renew before expiry.**
3. **MANDATORY HTTPS endpoint with valid, non-self-signed TLS certificate.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-graph-webhooks-change-notifications.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-graph-webhooks-change-notifications.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Deploy robust webhook receivers to process real-time Graph change notifications. Pair webhook signals with Delta query execution.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-graph-webhooks-change-notifications.**
- **Unmonitored runtime execution without telemetry in copilot-graph-webhooks-change-notifications.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "graph-webhooks"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
