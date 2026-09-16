---
name: copilot-rbac-least-privilege-scoping
description: "Scoping Graph application vs delegated permissions to absolute least-privilege roles."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Least Privilege Security in Microsoft Graph API - Microsoft Identity"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["rbac-least-privilege", "graph-permission-scoping", "delegated-vs-application", "least-privilege-access"]
---

# copilot-rbac-least-privilege-scoping
> Based on **Least Privilege Security in Microsoft Graph API - Microsoft Identity** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Principle of Least Privilege: Request delegated permissions (e.g. `Mail.Read`) over application permissions (`Mail.Read.All`).**
2. **Dynamic Consent: Request high-privilege scopes incrementally when the user triggers the specific feature.**
3. **NEVER grant `Directory.ReadWrite.All` or `Files.ReadWrite.All` to custom agent service principals.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-rbac-least-privilege-scoping.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-rbac-least-privilege-scoping actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Audit and constrain Microsoft Graph permissions to the exact minimum scopes necessary for agent operation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-rbac-least-privilege-scoping.**
- **Unmonitored runtime execution without telemetry in copilot-rbac-least-privilege-scoping.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "rbac-least-privilege"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
