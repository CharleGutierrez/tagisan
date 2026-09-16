---
name: copilot-servicenow-incident-cmdb-connector
description: "ServiceNow Table API integration for IT Service Management (incidents, change requests, CMDB)."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "ServiceNow ITSM Automation with Microsoft Copilot - Tim Woodruff"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["servicenow-copilot", "itsm-automation", "table-api-servicenow", "incident-management-copilot"]
---

# copilot-servicenow-incident-cmdb-connector
> Based on **ServiceNow ITSM Automation with Microsoft Copilot - Tim Woodruff** (Cluster 15)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Table API Binding: Interact with `incident`, `change_request`, and `cmdb_ci` tables via REST API.**
2. **OAuth2 Token Flow: Authenticate via ServiceNow OAuth Provider with scoped client credentials.**
3. **ALWAYS map incident urgency and impact to valid ServiceNow choice values.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-servicenow-incident-cmdb-connector.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-servicenow-incident-cmdb-connector actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Automate IT service management workflows: create, query, and escalate ServiceNow incidents directly within Copilot.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-servicenow-incident-cmdb-connector.**
- **Unmonitored runtime execution without telemetry in copilot-servicenow-incident-cmdb-connector.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "servicenow-copilot"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
