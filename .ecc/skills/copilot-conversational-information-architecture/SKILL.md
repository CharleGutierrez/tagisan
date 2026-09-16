---
name: copilot-conversational-information-architecture
description: "Structuring chat responses with scannable headers, bullet hierarchy, and bold takeaways."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Conversational AI: Design and Engineering - Cathy Pearl"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["conversational-info-architecture", "scannable-chat-responses", "response-hierarchy", "conversational-ux"]
---

# copilot-conversational-information-architecture
> Based on **Conversational AI: Design and Engineering - Cathy Pearl** (Cluster 13)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Information Hierarchy: 1) One-sentence summary answer, 2) Bulleted key takeaways, 3) Detailed supporting context.**
2. **Scannability Invariant: Use bold lead-ins on bullet points to permit rapid executive skimming.**
3. **NEVER emit wall-of-text paragraphs exceeding 4 sentences in chat streams.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-conversational-information-architecture.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-conversational-information-architecture actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure Copilot chat outputs with crisp information architecture, bulleted takeaways, and bold visual anchors.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-conversational-information-architecture.**
- **Unmonitored runtime execution without telemetry in copilot-conversational-information-architecture.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "conversational-info-architecture"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
