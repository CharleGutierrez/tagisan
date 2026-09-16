---
name: copilot-azure-ai-search-hybrid-hnsw
description: "Azure AI Search vector indexing using HNSW, scalar quantization, and BM25 hybrid search."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Azure AI Search: Enterprise Vector and Hybrid Retrieval - Liam Cavanagh"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["azure-ai-search-hybrid", "hnsw-vector-indexing", "bm25-hybrid-retrieval", "scalar-quantization"]
---

# copilot-azure-ai-search-hybrid-hnsw
> Based on **Azure AI Search: Enterprise Vector and Hybrid Retrieval - Liam Cavanagh** (Cluster 12)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Hybrid Fusion: Combine BM25 full-text search scores with HNSW vector cosine similarity via RRF.**
2. **Scalar Quantization: Compress float32 vectors to int8 to reduce memory footprint by 75% with negligible accuracy loss.**
3. **MANDATORY index schema configuration defining vector dimensions (e.g. 3072 for text-embedding-3-large).**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-azure-ai-search-hybrid-hnsw.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-azure-ai-search-hybrid-hnsw.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Deploy hybrid search indexes in Azure AI Search combining BM25 keyword matching and HNSW vector similarity.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-azure-ai-search-hybrid-hnsw.**
- **Unmonitored runtime execution without telemetry in copilot-azure-ai-search-hybrid-hnsw.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "azure-ai-search-hybrid"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
