---
name: copilot-contextual-compression-retrieval
description: "Compressing retrieved document chunks to isolate high-density relevant facts."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Retrieval-Augmented Generation: Systems, Architectures and Optimizations - Patrick Lewis"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["contextual-compression", "chunk-compression", "fact-density-extraction", "irrelevant-token-pruning"]
---

# copilot-contextual-compression-retrieval
> Based on **Retrieval-Augmented Generation: Systems, Architectures and Optimizations - Patrick Lewis** (Cluster 12)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Compression Technique: Filter retrieved document text through a fast extractor model to strip irrelevant filler.**
2. **Token Savings: Reduce retrieved context token consumption by 50-70% while boosting factual density.**
3. **ALWAYS maintain original document citation references on compressed passages.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-contextual-compression-retrieval.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-contextual-compression-retrieval actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Deploy contextual compression passes over retrieved chunks to maximize signal-to-noise ratio in model context windows.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-contextual-compression-retrieval.**
- **Unmonitored runtime execution without telemetry in copilot-contextual-compression-retrieval.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "contextual-compression"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
