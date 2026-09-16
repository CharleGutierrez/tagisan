---
name: copilot-context-grounding-citations
description: "Formulating prompts to enforce strict citation anchors ([doc1], [doc2]) and ground-truth verification."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Designing Transparent Citations and Attribution in AI Responses - Microsoft UX"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["context-grounding-citations", "citation-anchors", "ground-truth-verification", "verifiable-attribution"]
---

# copilot-context-grounding-citations
> Based on **Designing Transparent Citations and Attribution in AI Responses - Microsoft UX** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Citation Syntax: Force model to suffix every factual assertion with exact document chunk IDs `[docX]`.**
2. **No Hallucinated Citations: STRICT_REJECT responses citing document IDs not present in retrieved context.**
3. **ALWAYS include direct web/SharePoint URLs in citation reference tables.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-context-grounding-citations.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-context-grounding-citations actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce strict citation schemas in system prompts to ensure every claim made by Copilot is auditable and verifiable.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-context-grounding-citations.**
- **Unmonitored runtime execution without telemetry in copilot-context-grounding-citations.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "context-grounding-citations"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
