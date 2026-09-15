---
name: agentic-robinson-graph-rag-knowledge
description: "Property graph models, graph traversal algorithms, Cypher queries, Knowledge Graph RAG, and multi-hop relationship reasoning across codebases."
triggers: ["robinson", "webber", "eifrem", "graph-databases", "graph-rag", "property-graphs", "cypher-queries", "multi-hop-retrieval"]
---

# agentic-robinson-graph-rag-knowledge
> Based on **Graph Databases: New Opportunities for Connected Data - Ian Robinson, Jim Webber & Emil Eifrem**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Labeled Property Graph Model: Nodes with labels and key-value properties connected by directed, typed relationships with properties.**
2. **Index-Free Adjacency: Each node directly references its adjacent neighbors, allowing O(1) traversal performance independent of total graph size.**
3. **Multi-Hop Graph RAG: Traversing 2-3 degrees of separation (Function -> Calls -> Dependency -> Version) to retrieve complete architectural context.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build a Graph RAG pipeline over codebase ASTs and dependency graphs. Use graph traversals to gather multi-hop context for complex refactorings.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Flat keyword search across disconnected files when understanding a bug requires walking call-graph paths.**
- **Unbounded breadth-first graph expansions that explode memory and retrieve irrelevant modules.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "robinson"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
