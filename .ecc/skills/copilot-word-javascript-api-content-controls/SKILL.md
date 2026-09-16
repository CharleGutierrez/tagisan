---
name: copilot-word-javascript-api-content-controls
description: "Word JavaScript API: manipulating paragraphs, content controls, inline styles, and OOXML bodies."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Word Document Generation and Templating with Office.js - Doug Mahugh"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["word-javascript-api", "word-content-controls", "ooxml-document-generation", "word-automation-js"]
---

# copilot-word-javascript-api-content-controls
> Based on **Word Document Generation and Templating with Office.js - Doug Mahugh** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Document DOM: Manipulate `context.document.body`, `contentControls`, and `tables` via Office.js.**
2. **Batch Synchronization: Call `await context.sync()` after batching DOM mutations.**
3. **MANDATORY cleanup of temporary content controls post-generation.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-word-javascript-api-content-controls.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-word-javascript-api-content-controls.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Automate rich document generation and templating in Microsoft Word using the Office.js JavaScript API.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-word-javascript-api-content-controls.**
- **Unmonitored runtime execution without telemetry in copilot-word-javascript-api-content-controls.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "word-javascript-api"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
