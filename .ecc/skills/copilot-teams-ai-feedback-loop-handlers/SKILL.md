---
name: copilot-teams-ai-feedback-loop-handlers
description: "Capturing user thumbs-up / thumbs-down feedback, citation chips, and telemetry dispatch."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "User Feedback Loops (Thumbs Up/Down) in Teams AI - Microsoft UX"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-ai-feedback-loops", "thumbs-up-down", "citation-chips", "user-feedback-handlers"]
---

# copilot-teams-ai-feedback-loop-handlers
> Based on **User Feedback Loops (Thumbs Up/Down) in Teams AI - Microsoft UX** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Feedback Handlers: Register `app.feedbackLoop` to capture user quality sentiment.**
2. **Citation Integration: Attach citation references (`citations: [...]`) to bot messages for verifiable grounding.**
3. **MANDATORY logging of negative feedback events to offline eval telemetry.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-ai-feedback-loop-handlers.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-ai-feedback-loop-handlers.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Equip custom engine agents with native feedback loops to capture user sentiment and drive continuous prompt refinement.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-feedback-loop-handlers.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-feedback-loop-handlers.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-ai-feedback-loops"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
