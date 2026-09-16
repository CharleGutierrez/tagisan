---
name: copilot-rag-query-expansion-hyde
description: "Hypothetical Document Embeddings (HyDE) and multi-query reformulation for Graph Search."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Hypothetical Document Embeddings (HyDE) and Query Expansion - Luyu Gao"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["query-expansion-hyde", "hypothetical-embeddings", "multi-query-reformulation", "graph-search-expansion"]
---

# copilot-rag-query-expansion-hyde
> Based on **Hypothetical Document Embeddings (HyDE) and Query Expansion - Luyu Gao** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **HyDE Method: Generate a synthetic ideal answer document; embed the synthetic document to search vector space.**
2. **Multi-Query Reformulation: Generate 3 diverse keyword queries to search inverted index (BM25) in parallel.**
3. **ALWAYS merge and deduplicate search results using Reciprocal Rank Fusion (RRF).**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-rag-query-expansion-hyde.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-rag-query-expansion-hyde actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enhance enterprise search recall by deploying HyDE and multi-query reformulation before querying the Semantic Index.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-rag-query-expansion-hyde.**
- **Unmonitored runtime execution without telemetry in copilot-rag-query-expansion-hyde.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "query-expansion-hyde"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
