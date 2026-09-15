---
name: agentic-goldberg-nn-nlp
description: "Continuous vector representations, dense embeddings, compositional semantics, feedforward and recurrent networks, and geometric embedding spaces."
triggers: ["goldberg", "neural-nlp", "dense-embeddings", "vector-representations", "compositional-semantics", "embedding-geometry"]
---

# agentic-goldberg-nn-nlp
> Based on **Neural Network Methods for Natural Language Processing - Yoav Goldberg**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Distributional Hypothesis: Words occurring in similar contexts share similar semantic representations in vector space.**
2. **Vector Cosine Similarity: sim(u, v) = (u . v) / (||u|| * ||v||), invariant to vector magnitude.**
3. **Compositionality: Representing complex phrases or code blocks as geometric compositions of their atomic token vectors.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Index codebases in high-dimensional vector spaces using semantic embeddings. Compute cosine similarity against user problem statements to retrieve relevant source files.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying solely on vector embeddings for exact identifier search where inverted lexical search is required.**
- **Comparing embeddings across heterogeneous vector spaces without shared alignment.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "goldberg"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
