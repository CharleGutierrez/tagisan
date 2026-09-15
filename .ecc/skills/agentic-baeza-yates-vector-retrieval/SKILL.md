---
name: agentic-baeza-yates-vector-retrieval
description: "Vector ranking models, probabilistic retrieval, index compression, evaluation metrics (MAP, NDCG), and query expansion algorithms."
triggers: ["baeza-yates", "ribeiro-neto", "modern-retrieval", "ndcg-ranking", "query-expansion", "vector-ranking", "probabilistic-retrieval"]
---

# agentic-baeza-yates-vector-retrieval
> Based on **Modern Information Retrieval - Ricardo Baeza-Yates & Berthier Ribeiro-Neto**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Normalized Discounted Cumulative Gain (NDCG): NDCG_p = DCG_p / IDCG_p, where DCG_p = sum_{i=1}^p (2^{rel_i} - 1) / log_2(i + 1).**
2. **Query Expansion: Augmenting short user queries with related synonyms, types, and compiler error signatures to improve retrieval recall.**
3. **Rank Fusion (RRF): Reciprocal Rank Fusion: RRF(d) = sum_{m in models} 1 / (k + rank_m(d)), merging disparate retrieval rankings.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Reciprocal Rank Fusion to combine lexical AST search, git blame logs, and vector embeddings. Score results using NDCG metrics.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming top-1 retrieval is always accurate; always feed top-K diversified context chunks.**
- **Evaluating search pipelines without standard ground-truth relevance benchmarks.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "baeza-yates"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
