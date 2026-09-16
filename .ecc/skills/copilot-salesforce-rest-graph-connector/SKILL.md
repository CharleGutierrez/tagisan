---
name: copilot-salesforce-rest-graph-connector
description: "Salesforce REST API integration, SOQL query mapping, and indexing Accounts/Opportunities in Graph."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Salesforce CRM and Microsoft 365 Copilot Interoperability - Phil Weinmeister"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["salesforce-copilot", "salesforce-graph-connector", "soql-query-mapping", "crm-opportunities-sync"]
---

# copilot-salesforce-rest-graph-connector
> Based on **Salesforce CRM and Microsoft 365 Copilot Interoperability - Phil Weinmeister** (Cluster 15)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **OAuth2 Web Server Flow: Authenticate via Salesforce Connected App with PKCE.**
2. **SOQL Safety: Sanitize user input before interpolating into SOQL queries to prevent SOQL injection.**
3. **MANDATORY field-level security (FLS) enforcement matching Salesforce permissions.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-salesforce-rest-graph-connector.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-salesforce-rest-graph-connector.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Index Salesforce CRM Accounts, Contacts, and Opportunities into Microsoft Copilot for unified enterprise search.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-salesforce-rest-graph-connector.**
- **Unmonitored runtime execution without telemetry in copilot-salesforce-rest-graph-connector.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "salesforce-copilot"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
