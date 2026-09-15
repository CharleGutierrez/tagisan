---
name: agentic-sowa-knowledge-representation
description: "Conceptual graphs, first-order logic semantics, ontologies, semantic networks, semantic ambiguity resolution, and knowledge graph mapping."
triggers: ["sowa", "knowledge-representation", "conceptual-graphs", "ontologies", "first-order-logic", "semantic-networks"]
---

# agentic-sowa-knowledge-representation
> Based on **Knowledge Representation: Logical, Philosophical, and Computational Foundations - John F. Sowa**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Conceptual Graph Invariant: Bipartite graph of Concept nodes and Conceptual Relation nodes with formal first-order logic mappings.**
2. **Ontological Commitment: Explicitly specifying the categories, relations, and invariants that exist within the software problem domain.**
3. **Knowledge Fusion: Merging disparate semantic schemas via graph unification and constraint consistency checking.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Construct formal domain ontologies for target codebases. Model relationships (implements, extends, calls, imports) as typed conceptual graphs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Unconstrained natural language summaries that introduce logical contradictions into the system's world model.**
- **Assuming isomorphic schemas across different microservices without explicit translation mappings.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sowa"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
