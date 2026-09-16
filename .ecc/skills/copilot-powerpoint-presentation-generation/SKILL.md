---
name: copilot-powerpoint-presentation-generation
description: "Automating PowerPoint slide creation via OpenXML, Marp Markdown, and Office JavaScript API."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "PowerPoint Automation with Office JavaScript API - Microsoft Docs"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["powerpoint-generation", "powerpoint-office-js", "marp-markdown-slides", "openxml-slides"]
---

# copilot-powerpoint-presentation-generation
> Based on **PowerPoint Automation with Office JavaScript API - Microsoft Docs** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Slide Hierarchy: Add slides, shape trees, text frames, and table layouts programmatically.**
2. **Layout Templates: Bind content to pre-defined slide masters to maintain brand design consistency.**
3. **MANDATORY validation of shape coordinates and text boundary wrapping.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-powerpoint-presentation-generation.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-powerpoint-presentation-generation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Automate corporate presentation synthesis from structured Markdown or JSON data using PowerPoint APIs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-powerpoint-presentation-generation.**
- **Unmonitored runtime execution without telemetry in copilot-powerpoint-presentation-generation.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "powerpoint-generation"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
