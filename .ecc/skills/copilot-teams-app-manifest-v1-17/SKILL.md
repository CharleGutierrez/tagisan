---
name: copilot-teams-app-manifest-v1-17
description: "Teams app manifest schema v1.16/v1.17, copilotAgents declarations, bot definitions, and delegated permission scopes."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-app-manifest", "manifest-json-v1-17", "copilotagents-declaration", "teams-manifest-schema"]
---

# copilot-teams-app-manifest-v1-17
> Based on **Microsoft Teams App Packaging and Lifecycle - Microsoft Docs** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Manifest Top-Level: `manifestVersion: 1.17`, `copilotAgents: { declarativeAgents: [...] }`.**
2. **Icon Assets: `color.png` (192x192) and `outline.png` (32x32) strictly matching transparent PNG requirements.**
3. **ALWAYS declare required Graph API delegated permission scopes in `webApplicationInfo`.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-app-manifest-v1-17.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-app-manifest-v1-17 actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Validate Teams app packages against schema v1.17 using Teams Toolkit CLI: `teamsapp validate` before publishing.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using JPEG or non-square icon files causing silent packaging rejections.**
- **Mismatched IDs between `manifest.json` and `declarativeAgent.json`.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-app-manifest"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
