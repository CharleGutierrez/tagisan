---
name: copilot-jira-cloud-confluence-rest-connector
description: "Atlassian Jira Cloud & Confluence REST v3 integration for issues, epics, and documentation sync."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Atlassian Ecosystem Integration with Microsoft 365 - Patrick Li"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["jira-confluence-copilot", "atlassian-rest-connector", "jira-issue-tracking", "confluence-knowledge-sync"]
---

# copilot-jira-cloud-confluence-rest-connector
> Based on **Atlassian Ecosystem Integration with Microsoft 365 - Patrick Li** (Cluster 15)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **REST v3 Contracts: Query Jira issues via JQL (`/rest/api/3/search`) and Confluence spaces via v2 API.**
2. **Atlassian Document Format (ADF): Parse and generate structured ADF payloads for issue descriptions.**
3. **MANDATORY handling of Atlassian cloud rate limits (100 req/min).**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-jira-cloud-confluence-rest-connector.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-jira-cloud-confluence-rest-connector.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Integrate Atlassian Jira and Confluence with Microsoft Copilot to query sprints, track tickets, and synthesize docs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-jira-cloud-confluence-rest-connector.**
- **Unmonitored runtime execution without telemetry in copilot-jira-cloud-confluence-rest-connector.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "jira-confluence-copilot"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
