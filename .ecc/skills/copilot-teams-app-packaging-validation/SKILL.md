---
name: copilot-teams-app-packaging-validation
description: "Teams package zip generation (color.png, outline.png, manifest.json) and automated schema validation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Teams App Packaging and Lifecycle - Microsoft Docs"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-app-packaging", "package-zip-generation", "teams-schema-validation", "teamsapp-package"]
---

# copilot-teams-app-packaging-validation
> Based on **Microsoft Teams App Packaging and Lifecycle - Microsoft Docs** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Zip Archive Structure: Exact zip bundle containing `manifest.json`, `color.png`, `outline.png`, and optional declarative agent JSON files.**
2. **Zero Compression Corruption: File headers in the ZIP archive must conform strictly to standard PKZip formatting.**
3. **MANDATORY schema validation against the official Microsoft Teams schema before upload.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-app-packaging-validation.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-app-packaging-validation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use automated validation scripts (`teamsapp validate --package appPackage/build/appPackage.dev.zip`) in continuous integration gates.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Including parent folder paths inside the ZIP archive, breaking manifest extraction.**
- **Mismatched image dimensions (e.g. non-32x32 outline icon) resulting in package rejection.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-app-packaging"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
