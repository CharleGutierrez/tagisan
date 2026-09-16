---
name: copilot-cross-system-identity-reconciliation
description: "Mapping user identity across Entra ID, SAP, Salesforce, and Jira for secure authorization trimming."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["cross-system-identity", "identity-reconciliation", "authorization-trimming", "federated-identity-mapping"]
---

# copilot-cross-system-identity-reconciliation
> Based on **Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci** (Cluster 15)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Identity Mapping Table: Maintain secure, audited mapping between Entra ID UserPrincipalName and external system IDs.**
2. **Security Trimming Invariant: Users MUST only see data they have permission to access in BOTH systems.**
3. **ALWAYS verify active account status in Entra ID before resolving external identity.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-cross-system-identity-reconciliation.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-cross-system-identity-reconciliation actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Reconcile user identities across disparate enterprise SaaS systems to enforce seamless security trimming in Copilot.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-cross-system-identity-reconciliation.**
- **Unmonitored runtime execution without telemetry in copilot-cross-system-identity-reconciliation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cross-system-identity"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
