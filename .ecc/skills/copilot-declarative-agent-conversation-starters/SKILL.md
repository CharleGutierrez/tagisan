---
name: copilot-declarative-agent-conversation-starters
description: "Designing conversation starters, prompt scaffolds, and multi-lingual localized string packages."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Conversational AI: Design and Engineering - Cathy Pearl"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["conversation-starters", "prompt-scaffolds", "localization-packages", "agent-ux-starters"]
---

# copilot-declarative-agent-conversation-starters
> Based on **Conversational AI: Design and Engineering - Cathy Pearl** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Conversation Starter Schema: Array of text prompts under `conversation_starters` in manifest.**
2. **Cognitive Load Reduction: Starters must be concrete, action-oriented, and demonstrate agent capabilities.**
3. **MANDATORY localization files (`localization.json`) when publishing to multi-regional tenant workforces.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-declarative-agent-conversation-starters.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-declarative-agent-conversation-starters.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Provide 3 to 6 high-value conversation starters representing canonical user journeys. Ensure starters trigger specific action plugins or document retrieval flows.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Generic starters like 'Help me' or 'What can you do?' that fail to showcase capabilities.**
- **Starters that trigger unsupported actions, immediately eroding user trust.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "conversation-starters"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
