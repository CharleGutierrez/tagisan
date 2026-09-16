---
name: copilot-graph-search-query-api
description: "Programmatic querying of Microsoft Search API (/search/query) for hybrid BM25 and vector semantic hits."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Deep Dive into Microsoft 365 Semantic Index - Satya Nadella et al."
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["graph-search-api", "search-query-api", "hybrid-bm25-search", "semantic-search-hits"]
---

# copilot-graph-search-query-api
> Based on **Deep Dive into Microsoft 365 Semantic Index - Satya Nadella et al.** (Cluster 4)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Entity Types: Query across message, chatMessage, driveItem, externalItem, listItem.**
2. **KQL Filtering: Support Keyword Query Language (KQL) expressions alongside natural language queries.**
3. **MANDATORY pagination handling via 'from' and 'size' parameters.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-graph-search-query-api.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-graph-search-query-api.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Execute unified enterprise search queries via `/search/query` to retrieve multi-workload grounded hits for Copilot reasoning.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-graph-search-query-api.**
- **Unmonitored runtime execution without telemetry in copilot-graph-search-query-api.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "graph-search-api"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
