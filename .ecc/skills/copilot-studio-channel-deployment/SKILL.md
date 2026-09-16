---
name: copilot-studio-channel-deployment
description: "Multi-channel publishing (Teams, Webchat, Outlook, Omnichannel, custom mobile applications)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Mastering Microsoft Copilot Studio - Robert Kaack"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["channel-deployment", "multi-channel-publishing", "teams-channel-copilot", "webchat-deployment"]
---

# copilot-studio-channel-deployment
> Based on **Mastering Microsoft Copilot Studio - Robert Kaack** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Omnichannel Distribution: Single bot core deployed across Microsoft Teams, Webchat, Power Pages, and mobile apps.**
2. **Channel Adaptation Invariant: Rich UI cards must gracefully downgrade to plain text on channels lacking Adaptive Card support.**
3. **MANDATORY channel security configuration (Direct Line token generation, trusted origins).**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-studio-channel-deployment.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-channel-deployment.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Configure channel-specific settings, ensuring security tokens are exchanged securely via backend server components.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Exposing raw Direct Line secret keys in client-side JavaScript.**
- **Sending complex interactive cards to SMS or email channels without plain-text fallbacks.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "channel-deployment"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
