---
name: arch-robinson-graph-databases
description: "Graph database architecture: Property graph data model, index-free adjacency, recursive traversal algorithms, and relationship-centric domain modeling."
triggers: ["robinson-graph", "graph-databases", "index-free-adjacency", "property-graph", "graph-traversal", "cypher-patterns", "relationship-first"]
---

# arch-robinson-graph-databases
> Based on **Graph Databases - Ian Robinson, Jim Webber, Emil Eifrem**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Queries traversing interconnected domain relationships beyond 2 hops must utilize index-free adjacency instead of recursive relational JOINs.**
2. **ALWAYS: Maintain directional typed relationships with first-class properties to eliminate costly associative junction tables.**
3. **NEVER: Perform full-graph global scans without index anchor entry points.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Model networks, permissions, social graphs, and bill-of-materials as property graphs. Anchor traversals at indexed nodes and traverse pointers in O(1) per edge.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Writing recursive SQL CTEs with 5+ JOINs for deeply nested graph navigation.**
- **Treating graph databases as simple key-value document stores.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-robinson-graph-databases"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
