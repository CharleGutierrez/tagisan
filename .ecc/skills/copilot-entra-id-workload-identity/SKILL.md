---
name: copilot-entra-id-workload-identity
description: "Service principals, managed identities, federated credentials, and certificate authentication."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["entra-id-workload-identity", "managed-identities", "federated-credentials", "service-principals"]
---

# copilot-entra-id-workload-identity
> Based on **Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Identity Architecture: Prefer User-Assigned Managed Identity over client secrets for Azure-hosted Copilot backends.**
2. **Certificate Credential: Use X.509 certificate credentials when authenticating service principals from on-premises.**
3. **NEVER store plain-text client secrets in environment variables or configuration files.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-entra-id-workload-identity.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-entra-id-workload-identity actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Secure backend Copilot services using Microsoft Entra ID workload identities and federated credentials.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-entra-id-workload-identity.**
- **Unmonitored runtime execution without telemetry in copilot-entra-id-workload-identity.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "entra-id-workload-identity"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
