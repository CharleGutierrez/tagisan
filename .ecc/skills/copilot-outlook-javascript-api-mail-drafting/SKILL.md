---
name: copilot-outlook-javascript-api-mail-drafting
description: "Outlook add-ins: drafting email replies, inspecting attachments, and appointment creation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Building Modern Outlook Web Add-ins - Andrew Coates"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["outlook-javascript-api", "outlook-mail-drafting", "item-send-events", "outlook-add-ins"]
---

# copilot-outlook-javascript-api-mail-drafting
> Based on **Building Modern Outlook Web Add-ins - Andrew Coates** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Mailbox API: Access active message via `Office.context.mailbox.item`.**
2. **Item-Send Hooks: Validate email contents and sensitivity labels before sending via OnSend handlers.**
3. **ALWAYS request appropriate mailbox permission levels (ReadItem vs ReadWriteItem).**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-outlook-javascript-api-mail-drafting.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-outlook-javascript-api-mail-drafting actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build modern Outlook Web Add-ins that assist users with context-aware email drafting, triage, and scheduling.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-outlook-javascript-api-mail-drafting.**
- **Unmonitored runtime execution without telemetry in copilot-outlook-javascript-api-mail-drafting.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "outlook-javascript-api"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
