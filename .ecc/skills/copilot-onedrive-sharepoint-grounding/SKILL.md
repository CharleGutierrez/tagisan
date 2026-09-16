---
name: copilot-onedrive-sharepoint-grounding
description: "Scoping SharePoint site collections, document libraries, OneDrive folders, and URLs in agent capabilities."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "SharePoint & OneDrive REST APIs with Microsoft Graph - Todd Baginski"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sharepoint-grounding", "onedrive-grounding", "site-collection-scoping", "declarative-agent-sources"]
---

# copilot-onedrive-sharepoint-grounding
> Based on **SharePoint & OneDrive REST APIs with Microsoft Graph - Todd Baginski** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **URL Scoping: Specify explicit SharePoint site collection or folder URLs in `OneDriveAndSharePoint` capability.**
2. **Permissions Invariant: The agent cannot view or search files that the calling user lacks permission to access.**
3. **MANDATORY inclusion of exact SharePoint URL strings matching tenant site topology.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-onedrive-sharepoint-grounding.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-onedrive-sharepoint-grounding.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Ground Declarative Agents to specific, curated SharePoint document libraries rather than broad tenant-wide searches to maximize precision.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Pointing grounding URLs to top-level tenant roots, causing high retrieval noise.**
- **Attempting to ground against external non-SharePoint web URLs in the SharePoint capability array.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sharepoint-grounding"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
