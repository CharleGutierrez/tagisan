---
name: copilot-continuous-access-evaluation-cae
description: "Handling CAE claims challenges (WWW-Authenticate: Bearer error="insufficient_claims")."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["continuous-access-evaluation", "cae-claims-challenge", "insufficient-claims-handling", "real-time-session-revocation"]
---

# copilot-continuous-access-evaluation-cae
> Based on **Identity and Access Management with Microsoft Entra ID - Vittorio Bertocci** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **CAE Mechanics: Entra ID revokes tokens in real time upon critical events (password change, user disablement).**
2. **Claims Challenge Parsing: Parse `claims` parameter from HTTP 401 response and redirect user to re-authenticate.**
3. **ALWAYS support CAE in all Microsoft Graph client SDK configurations.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-continuous-access-evaluation-cae.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-continuous-access-evaluation-cae actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Handle Continuous Access Evaluation claims challenges gracefully, triggering immediate re-authentication upon session revocation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-continuous-access-evaluation-cae.**
- **Unmonitored runtime execution without telemetry in copilot-continuous-access-evaluation-cae.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "continuous-access-evaluation"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
