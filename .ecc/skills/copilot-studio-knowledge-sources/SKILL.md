---
name: copilot-studio-knowledge-sources
description: "Integrating SharePoint, Dataverse, Public Web, and unstructured files as grounded knowledge sources."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Enterprise Knowledge Grounding in Copilot Studio - Alex Simons"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-studio-knowledge", "knowledge-sources", "dataverse-grounding", "sharepoint-sources"]
---

# copilot-studio-knowledge-sources
> Based on **Enterprise Knowledge Grounding in Copilot Studio - Alex Simons** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Heterogeneous Ingestion: Direct binding of SharePoint URLs, Dataverse tables, uploaded PDFs/DOCX, and public URLs.**
2. **Index Refresh Invariant: Knowledge sources must specify automated indexing intervals or webhook-driven updates.**
3. **MANDATORY authentication configuration for internal SharePoint and Dataverse sources.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-studio-knowledge-sources.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-knowledge-sources.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Curate and partition knowledge sources into dedicated topics to prevent semantic cross-talk and maximize retrieval accuracy.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Uploading outdated PDF handbooks containing conflicting HR policies.**
- **Exposing unsecured internal Dataverse tables without row-level security.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-studio-knowledge"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
