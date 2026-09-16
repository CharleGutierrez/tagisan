---
name: copilot-power-apps-copilot-studio-embedding
description: "Embedding Copilot Studio bots into Canvas Apps and Model-Driven Apps with context passing."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Learn Microsoft Power Platform and Copilot - Robert Kaack"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-apps-copilot", "copilot-canvas-apps", "model-driven-copilot", "copilot-embedding"]
---

# copilot-power-apps-copilot-studio-embedding
> Based on **Learn Microsoft Power Platform and Copilot - Robert Kaack** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Context Passing: Pass active record ID and user context into embedded Copilot controls.**
2. **UI Integration: Embed Copilot sidecar panels without occluding core transactional form controls.**
3. **ALWAYS honor record-level security permissions defined in Dataverse.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-power-apps-copilot-studio-embedding.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-power-apps-copilot-studio-embedding actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Embed contextual Copilot assistants into Power Apps to assist users with real-time data entry and validation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-power-apps-copilot-studio-embedding.**
- **Unmonitored runtime execution without telemetry in copilot-power-apps-copilot-studio-embedding.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-apps-copilot"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
