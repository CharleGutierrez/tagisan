---
name: copilot-teams-ai-custom-engine-rag
description: "Grounding custom engine agent prompts with vector search results and dynamic system messages."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Retrieval-Augmented Generation: Systems, Architectures and Optimizations - Patrick Lewis"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["custom-engine-rag", "teams-ai-rag", "vector-search-grounding", "dynamic-system-messages"]
---

# copilot-teams-ai-custom-engine-rag
> Based on **Retrieval-Augmented Generation: Systems, Architectures and Optimizations - Patrick Lewis** (Cluster 5)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **RAG Pipeline: User Query -> Embedding Generation -> Vector Retrieval -> Reranking -> System Prompt Augmentation.**
2. **Context Token Limit: Truncate retrieved chunks strictly within prompt budget (e.g., 4000 tokens).**
3. **ALWAYS instruct model to cite retrieved chunk identifiers in answers.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-ai-custom-engine-rag.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-ai-custom-engine-rag actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Ground Teams AI agents with enterprise data retrieved from Azure AI Search or Qdrant, formatting context into structured prompts.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-teams-ai-custom-engine-rag.**
- **Unmonitored runtime execution without telemetry in copilot-teams-ai-custom-engine-rag.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "custom-engine-rag"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
