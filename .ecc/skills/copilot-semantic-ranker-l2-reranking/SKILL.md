---
name: copilot-semantic-ranker-l2-reranking
description: "Applying Azure AI Search Semantic Ranker for L2 cross-encoder reranking of retrieved chunks."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Advanced Reranking Algorithms for Information Retrieval - Omar Khattab"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["semantic-ranker-l2", "cross-encoder-reranking", "azure-semantic-ranking", "rerank-relevance-boost"]
---

# copilot-semantic-ranker-l2-reranking
> Based on **Advanced Reranking Algorithms for Information Retrieval - Omar Khattab** (Cluster 12)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Two-Stage Retrieval: Stage 1 retrieves top 50 candidates via Hybrid search; Stage 2 applies deep transformer cross-encoder.**
2. **Semantic Captions: Extract and highlight verbatim relevant passages (`@search.captions`) for direct citation.**
3. **MANDATORY evaluation of Semantic Score threshold (>1.5) before passing chunks to LLM.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-semantic-ranker-l2-reranking.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-semantic-ranker-l2-reranking.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Integrate Azure AI Search Semantic Ranker into retrieval pipelines to elevate high-relevance chunks and eliminate noise.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-semantic-ranker-l2-reranking.**
- **Unmonitored runtime execution without telemetry in copilot-semantic-ranker-l2-reranking.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "semantic-ranker-l2"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
