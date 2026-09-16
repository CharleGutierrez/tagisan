---
name: copilot-ragas-eval-rag-triad
description: "Evaluating RAG pipelines using RAGAS: Faithfulness, Answer Relevance, Context Precision, Context Recall."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "The RAG Triad: Context Relevance, Groundedness, and Answer Relevance - TruLens Engineering"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["ragas-eval-triad", "faithfulness-metric", "answer-relevance", "context-precision-recall"]
---

# copilot-ragas-eval-rag-triad
> Based on **The RAG Triad: Context Relevance, Groundedness, and Answer Relevance - TruLens Engineering** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **RAG Triad Metrics: 1) Context Relevance (retrieval quality), 2) Groundedness/Faithfulness (no hallucination), 3) Answer Relevance.**
2. **Quantitative Scoring: Automate calculation of 0.0 to 1.0 scores across test suites.**
3. **MANDATORY failure threshold: Fail CI/CD build if average faithfulness drops below 0.85.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-ragas-eval-rag-triad.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-ragas-eval-rag-triad.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Benchmark and continuously audit enterprise Copilot RAG pipelines using RAGAS and TruLens evaluation frameworks.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-ragas-eval-rag-triad.**
- **Unmonitored runtime execution without telemetry in copilot-ragas-eval-rag-triad.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "ragas-eval-triad"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
