---
name: copilot-document-chunking-token-strategies
description: "Chunking strategies: semantic boundary chunking, sliding window with overlap, and markdown splitting."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Document Parsing, Chunking and Metadata Enrichment - Denny Britz"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["document-chunking-strategies", "semantic-boundary-chunking", "sliding-window-overlap", "markdown-splitting"]
---

# copilot-document-chunking-token-strategies
> Based on **Document Parsing, Chunking and Metadata Enrichment - Denny Britz** (Cluster 12)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Chunk Sizing: Target 400-800 tokens per chunk with 10-15% token overlap between consecutive chunks.**
2. **Boundary Respect: Split at natural markdown boundaries (headers, paragraphs, tables); NEVER split mid-sentence or mid-table.**
3. **ALWAYS preserve document title, section breadcrumbs, and page numbers in chunk metadata.**
4. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-document-chunking-token-strategies actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement intelligent, document-structure-aware chunking pipelines that preserve semantic context across chunk boundaries.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-document-chunking-token-strategies.**
- **Unmonitored runtime execution without telemetry in copilot-document-chunking-token-strategies.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "document-chunking-strategies"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
