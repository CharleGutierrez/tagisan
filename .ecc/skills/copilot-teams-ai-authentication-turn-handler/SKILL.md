---
name: copilot-teams-ai-authentication-turn-handler
description: "TeamsBot SSO, OAuthPrompt, and silent token acquisition in Teams AI Library turns."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-ai-auth-handler", "teamsbot-sso", "oauth-prompt", "silent-token-acquisition"]
---

# copilot-teams-ai-authentication-turn-handler
> Based on **Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **SSO Flow: Request user token silently via Teams SSO; fallback to OAuthCard dialog if consent is needed.**
2. **Token Validation: Validate signature, issuer, audience, and scope before trusting token claims.**
3. **NEVER pass unvalidated client tokens to downstream internal APIs.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-ai-authentication-turn-handler.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-ai-authentication-turn-handler actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement seamless single sign-on authentication in Teams AI agents, acquiring scoped Graph tokens silently on behalf of users.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-authentication-turn-handler.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-authentication-turn-handler.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-ai-auth-handler"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
