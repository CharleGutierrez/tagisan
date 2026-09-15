---
name: agentic-malkov-hnsw-vector-indexing
description: "Hierarchical Navigable Small World (HNSW) graphs, skip-list topology, logarithmic search complexity, edge pruning heuristics, and vector index scaling."
triggers: ["malkov", "yashunin", "hnsw-graphs", "approximate-nearest-neighbors", "ann-search", "vector-indexing", "skip-list-topology"]
---

# agentic-malkov-hnsw-vector-indexing
> Based on **Efficient and Robust Approximate Nearest Neighbor Search using HNSW Graphs - Yu. A. Malkov & D. A. Yashunin**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Hierarchical Multilayer Graph: Multi-layer structure where top layers contain long-range skip edges and layer 0 contains dense local connectivity.**
2. **Logarithmic Search Complexity: Search navigates greedy local minima across layers with average time complexity O(log N).**
3. **Heuristic Edge Selection: Balances distance to candidates with angular diversity to prevent clustering and maintain navigability.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Configure HNSW index parameters (M, efConstruction, efSearch) for sub-10ms similarity queries across millions of code embeddings in local vector stores.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using brute-force flat L2 search in production, causing unacceptable latency as codebase grows.**
- **Setting efSearch too low, degrading recall below acceptable thresholds for critical code retrieval.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "malkov"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
