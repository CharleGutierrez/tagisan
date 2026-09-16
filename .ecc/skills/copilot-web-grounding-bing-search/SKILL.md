---
name: copilot-web-grounding-bing-search
description: "Configuring Bing search grounding in Declarative Agents with domain restrictions and citation parsing."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["web-grounding", "bing-search-grounding", "domain-restrictions", "citation-parsing"]
---

# copilot-web-grounding-bing-search
> Based on **Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Web Grounding Capability: `capabilities: [{ name: 'WebSearch' }]`.**
2. **Citation Synthesis: Web search results automatically synthesize footnotes and URL citations.**
3. **ALWAYS enforce domain filtering when proprietary or industry-specific sources must be prioritized.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-web-grounding-bing-search.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-web-grounding-bing-search actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enable WebSearch for agents requiring real-time external data (market data, regulatory changes, public API docs). Pair with clear citation extraction directives.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Enabling WebSearch for purely confidential internal agents, risking external information contamination.**
- **Permitting ungrounded web search without domain filtering for sensitive regulatory guidance.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "web-grounding"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
