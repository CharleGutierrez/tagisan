---
name: copilot-graph-connectors-schema-registration
description: "Registering external connections, defining property schemas, and search item indexing."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Building Microsoft Graph Connectors - Microsoft Press"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["graph-connectors", "connector-schema-registration", "external-connections", "search-item-indexing"]
---

# copilot-graph-connectors-schema-registration
> Based on **Building Microsoft Graph Connectors - Microsoft Press** (Cluster 4)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Schema Immutability: Connector property types cannot be altered once registered; additions require schema extension.**
2. **Semantic Annotations: Mark properties with 'isSearchable', 'isQueryable', 'isRetrievable', 'labels'.**
3. **ALWAYS assign aliases and semantic labels (title, url, iconUrl).**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-graph-connectors-schema-registration.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-graph-connectors-schema-registration actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Define external connection schemas with exact semantic annotations to ensure external data is fully searchable by Microsoft 365 Copilot.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-graph-connectors-schema-registration.**
- **Unmonitored runtime execution without telemetry in copilot-graph-connectors-schema-registration.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "graph-connectors"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
