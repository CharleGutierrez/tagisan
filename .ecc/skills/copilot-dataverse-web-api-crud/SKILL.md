---
name: copilot-dataverse-web-api-crud
description: "Querying and mutating Dataverse entities via Web API, deep insert, and alternate keys."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Dataverse Architecture and Best Practices - Julie Yack"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["dataverse-web-api", "dataverse-crud", "deep-insert-dataverse", "alternate-keys"]
---

# copilot-dataverse-web-api-crud
> Based on **Microsoft Dataverse Architecture and Best Practices - Julie Yack** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **OData v4 Protocol: Execute CRUD operations against `/api/data/v9.2/` using standard OData headers.**
2. **Optimistic Concurrency: Use `If-Match: ETag` to prevent mid-air collision updates.**
3. **MANDATORY pagination handling for record sets exceeding 5,000 items.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-dataverse-web-api-crud.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-dataverse-web-api-crud.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Integrate directly with Microsoft Dataverse Web API for high-throughput batch operations and deep relational inserts.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-dataverse-web-api-crud.**
- **Unmonitored runtime execution without telemetry in copilot-dataverse-web-api-crud.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "dataverse-web-api"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
