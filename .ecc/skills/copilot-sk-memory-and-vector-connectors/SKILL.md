---
name: copilot-sk-memory-and-vector-connectors
description: "Semantic memory, vector store abstractions (Azure AI Search, Qdrant, Chroma), and embedding generation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Vector Memory and Text Embeddings in Semantic Kernel - Mark Wallace"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sk-memory-connectors", "vector-store-abstractions", "qdrant-azure-search-sk", "text-embeddings-sk"]
---

# copilot-sk-memory-and-vector-connectors
> Based on **Vector Memory and Text Embeddings in Semantic Kernel - Mark Wallace** (Cluster 6)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Vector Store Record: Define strongly typed record models annotated with `[VectorStoreRecordKey]`, `[VectorStoreRecordData]`.**
2. **Distance Metrics: Enforce cosine similarity or dot product consistent with embedding model specifications.**
3. **ALWAYS batch embedding generation when indexing multiple documents.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-sk-memory-and-vector-connectors.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-sk-memory-and-vector-connectors actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Store and search vector representations of enterprise documents using Semantic Kernel's pluggable vector store interfaces.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-sk-memory-and-vector-connectors.**
- **Unmonitored runtime execution without telemetry in copilot-sk-memory-and-vector-connectors.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sk-memory-connectors"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
