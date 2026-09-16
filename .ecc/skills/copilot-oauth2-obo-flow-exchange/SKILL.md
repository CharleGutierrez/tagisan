---
name: copilot-oauth2-obo-flow-exchange
description: "On-Behalf-Of (OBO) token exchange between Teams Bot, mid-tier APIs, and Microsoft Graph."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["oauth2-obo-flow", "on-behalf-of-exchange", "token-exchange-graph", "delegated-user-tokens"]
---

# copilot-oauth2-obo-flow-exchange
> Based on **Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **OBO Protocol: Mid-tier service exchanges incoming user assertion token for downstream Graph API token via `/token` endpoint.**
2. **Scope Enforcement: Request only the minimal downstream scopes required for the specific user turn.**
3. **MANDATORY caching of downstream OBO tokens in distributed cache with token expiration TTL.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-oauth2-obo-flow-exchange.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-oauth2-obo-flow-exchange.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement secure OAuth2 On-Behalf-Of token exchanges across multi-tier Copilot agent architectures.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-oauth2-obo-flow-exchange.**
- **Unmonitored runtime execution without telemetry in copilot-oauth2-obo-flow-exchange.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "oauth2-obo-flow"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
