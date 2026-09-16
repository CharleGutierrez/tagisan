---
name: copilot-teams-task-modules-dialogs
description: "Launching interactive modal dialogs (Teams Dialogs / Task Modules) from card buttons."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-task-modules", "teams-dialogs", "modal-dialogs-teams", "taskmodule-fetch"]
---

# copilot-teams-task-modules-dialogs
> Based on **Microsoft Teams App Packaging and Lifecycle - Microsoft Docs** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Modal Dialog Contract: Respond to `task/fetch` with task info containing web URL or embedded Adaptive Card.**
2. **Result Handling: Process modal submit events via `task/submit` and update parent chat card.**
3. **MANDATORY modal sizing definition (Small, Medium, Large) matching content requirements.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-task-modules-dialogs.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-task-modules-dialogs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Launch focused modal dialogs from Copilot cards to handle complex multi-field forms without cluttering the chat history.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-task-modules-dialogs.**
- **Unmonitored runtime execution without telemetry in copilot-teams-task-modules-dialogs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-task-modules"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
