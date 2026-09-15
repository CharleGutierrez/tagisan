---
name: agentic-manning-information-retrieval
description: "Inverted indices, BM25 scoring, TF-IDF vector space model, cosine similarity, precision/recall curves, and text normalization."
triggers: ["manning", "raghavan", "schutze", "information-retrieval", "inverted-index", "bm25-scoring", "tf-idf", "precision-recall"]
---

# agentic-manning-information-retrieval
> Based on **Introduction to Information Retrieval - Christopher D. Manning, Prabhakar Raghavan & Hinrich Schütze**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **BM25 Scoring Formula: score(D, Q) = sum_{i=1}^n IDF(q_i) * (f(q_i, D) * (k_1 + 1)) / (f(q_i, D) + k_1 * (1 - b + b * (|D| / avgdl))).**
2. **Inverted Index Invariant: O(1) post-list lookup mapping term tokens directly to document frequency and document IDs.**
3. **Precision vs Recall Trade-off: Precision = TP / (TP + FP); Recall = TP / (TP + FN); optimize F1 score for code search.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use hybrid search combining BM25 keyword matching (for exact variable/function names) with vector search (for conceptual semantic queries).

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using vector similarity alone to find exact identifier definitions (e.g. `UserAuthenticationHandler`).**
- **Failing to normalize code tokens (casing, camelCase splitting), leading to index misses.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "manning"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
