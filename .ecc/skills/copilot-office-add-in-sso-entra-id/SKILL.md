---
name: copilot-office-add-in-sso-entra-id
description: "Single Sign-On (SSO) in Office Add-ins with Office.auth.getAccessToken() and backend token exchange."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Single Sign-On (SSO) in Office Add-ins - Microsoft Identity"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["office-sso", "office-auth-sso", "entra-id-office-sso", "bootstrap-token-exchange"]
---

# copilot-office-add-in-sso-entra-id
> Based on **Single Sign-On (SSO) in Office Add-ins - Microsoft Identity** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **SSO Flow: Call `Office.auth.getAccessToken()` to retrieve bootstrap token without popup prompting.**
2. **Backend Token Exchange: Exchange bootstrap token via OAuth2 On-Behalf-Of flow for Graph API access.**
3. **ALWAYS handle error 13003 (consent needed) by launching fallback interactive dialog.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-office-add-in-sso-entra-id.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-office-add-in-sso-entra-id actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement robust Single Sign-On in Office Add-ins, securing seamless access to tenant Graph resources.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-office-add-in-sso-entra-id.**
- **Unmonitored runtime execution without telemetry in copilot-office-add-in-sso-entra-id.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "office-sso"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
