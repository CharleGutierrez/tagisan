---
name: copilot-office-add-in-unified-manifest
description: "Modern unified manifest (manifest.json) for Office Add-ins vs XML manifest migration."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["office-unified-manifest", "manifest-json-office", "xml-to-json-manifest", "office-add-in-spec"]
---

# copilot-office-add-in-unified-manifest
> Based on **Microsoft Teams App Packaging and Lifecycle - Microsoft Docs** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Unified Schema: Transition from legacy XML manifests to Microsoft 365 unified manifest schema v1.17+.**
2. **Cross-Platform Compatibility: Single package targets Teams, Outlook, Word, Excel, PowerPoint.**
3. **MANDATORY specification of `extensions` array defining custom ribbon buttons and task panes.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-office-add-in-unified-manifest.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-office-add-in-unified-manifest.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Package and deploy Office Add-ins using the modern unified JSON manifest schema across Microsoft 365 hosts.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-office-add-in-unified-manifest.**
- **Unmonitored runtime execution without telemetry in copilot-office-add-in-unified-manifest.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "office-unified-manifest"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
