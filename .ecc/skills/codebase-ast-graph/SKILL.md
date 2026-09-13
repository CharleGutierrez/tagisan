---
name: codebase-ast-graph
description: AST codebase knowledge graph construction, polyglot symbol indexing, call-graph traversal, and blast-radius impact analysis
---

# Codebase AST Knowledge Graph & Blast-Radius Engine

## Core Graph Principles
1. **Polyglot Symbol Extraction**:
   - Extract definitions (`fn`, `def`, `func`, `struct`, `interface`, `class`, `trait`, `mod`, `type`) with exact line/col boundaries and signatures.
   - Record relationships: `Calls`, `Defines`, `Implements`, `Imports`, and `References`.
2. **Centrality & Hub Identification**:
   - Calculate in-degree centrality to identify architectural hubs that require heightened regression testing.
3. **Transitive Blast-Radius Analysis**:
   - Traverse incoming call and reference edges using breadth-first search to find all affected dependents.
   - Classify risk:
     - **Low** (0-2 dependents): Localized refactoring.
     - **Medium** (3-5 dependents): Multi-call-site verification.
     - **High** (6-12 dependents): Module-wide integration testing.
     - **Critical** (13+ dependents or foundational interfaces): Deprecation cycle and backward-compatible adapter required.
4. **Tool Integration**:
   - Use `tgs graph stats`, `tgs graph symbol`, `tgs graph callers`, `tgs graph callees`, `tgs graph blast-radius`, and `tgs graph export`.
   - Autonomous agents use `query_code_graph` and `calculate_blast_radius`.
