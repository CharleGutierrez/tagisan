---
name: copilot-graph-rag-knowledge-graphs
description: "Extracting entity-relation triplets from enterprise documents for Graph-based RAG traversal."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Graph RAG: Unlocking Knowledge Graphs with LLMs - Darren Edge et al. (Microsoft Research)"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["graph-rag-knowledge", "entity-relation-triplets", "knowledge-graph-rag", "hierarchical-summarization"]
---

# copilot-graph-rag-knowledge-graphs
> Based on **Graph RAG: Unlocking Knowledge Graphs with LLMs - Darren Edge et al. (Microsoft Research)** (Cluster 12)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Graph Extraction: Prompt LLMs to extract Entities, Relations, and Claims into a property graph.**
2. **Community Summarization: Perform hierarchical Leiden community detection to generate macro-level dataset summaries.**
3. **MANDATORY support for global queries ('What are the major themes across all project reports?').**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-graph-rag-knowledge-graphs.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-graph-rag-knowledge-graphs.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Microsoft Research GraphRAG pipelines to answer broad, thematic enterprise questions that defeat traditional vector search.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-graph-rag-knowledge-graphs.**
- **Unmonitored runtime execution without telemetry in copilot-graph-rag-knowledge-graphs.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "graph-rag-knowledge"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
