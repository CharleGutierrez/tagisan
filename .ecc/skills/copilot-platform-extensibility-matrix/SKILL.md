---
name: copilot-platform-extensibility-matrix
description: "Declarative Agents vs Custom Engine Agents vs Plugins selection architecture, execution boundaries, and licensing prerequisites."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-platform-extensibility", "extensibility-matrix", "declarative-vs-custom-engine", "copilot-plugins-architecture"]
---

# copilot-platform-extensibility-matrix
> Based on **Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Extensibility Tiers: Declarative Agents run within Microsoft 365 Copilot host context; Custom Engine Agents run standalone on Azure Bot Framework / Teams AI Library.**
2. **Execution Invariant: NEVER route tenant-sensitive enterprise requests to ungrounded external plugins without explicit OAuth2 administrative consent.**
3. **ALWAYS enforce user-scoped identity delegation using Entra ID On-Behalf-Of (OBO) flow for downstream API calls.**
4. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-platform-extensibility-matrix actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
ALWAYS evaluate requirements against the 3-tier matrix: 1) Declarative for grounded M365 context, 2) Plugins for external REST actions, 3) Custom Engine for full LLM orchestration control. MANDATORY 30s timeout on external connector invocations.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Selecting Custom Engine Agents for tasks purely requiring SharePoint/OneDrive retrieval, wasting infra overhead.**
- **Bypassing tenant boundary isolation by hardcoding ambient service principal tokens.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-platform-extensibility"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
