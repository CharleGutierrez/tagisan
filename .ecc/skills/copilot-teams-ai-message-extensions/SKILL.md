---
name: copilot-teams-ai-message-extensions
description: "Search-based and action-based Message Extensions, link unfurling (app.messageExtensions.handle)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Search and Action Message Extensions with AI - Waldek Mastykarz"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-message-extensions", "link-unfurling", "search-message-extension", "teams-ai-extensions"]
---

# copilot-teams-ai-message-extensions
> Based on **Search and Action Message Extensions with AI - Waldek Mastykarz** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Extension Types: Search extensions (query catalog), Action extensions (modal forms), Link unfurling (rich preview).**
2. **Response Format: Return `MessagingExtensionResponse` containing Adaptive Card or thumbnail preview attachments.**
3. **MANDATORY token verification in message extension invocations.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-ai-message-extensions.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-ai-message-extensions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement AI-powered Message Extensions allowing users to search, trigger, and insert rich Copilot artifacts directly into chats.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-message-extensions.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-message-extensions.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-message-extensions"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
