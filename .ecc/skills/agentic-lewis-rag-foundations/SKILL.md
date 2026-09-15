---
name: agentic-lewis-rag-foundations
description: "Dense passage retrieval, parametric vs non-parametric memory, cross-entropy loss over retrieved docs, and hybrid generation architecture."
triggers: ["lewis", "rag-foundations", "retrieval-augmented-generation", "dense-passage-retrieval", "non-parametric-memory", "hybrid-retrieval"]
---

# agentic-lewis-rag-foundations
> Based on **Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks - Patrick Lewis et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **RAG Generation Probability: P(y | x) = sum_{z in top-k} P(z | x) * P(y | x, z), marginalizing over retrieved document passages z.**
2. **Dual-Memory Paradigm: Parametric memory (frozen LLM neural weights) augmented with non-parametric memory (vector database of source documents).**
3. **Context Chunk Size Optimization: Finding the balance between semantic completeness (large chunks) and embedding retrieval precision (small chunks).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Augment agent generation with dense vector retrieval over codebase docs and history. Always cite specific file and line-number references for retrieved context.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying purely on parametric memory for internal codebase APIs, generating hallucinated methods.**
- **Injecting massive irrelevantly retrieved chunks that pollute context and cause hallucination.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "lewis"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
