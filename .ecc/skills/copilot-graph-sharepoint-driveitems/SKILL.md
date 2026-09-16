---
name: copilot-graph-sharepoint-driveitems
description: "Manipulating SharePoint Document Libraries, file download/upload chunks, and permissions inspection."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "SharePoint & OneDrive REST APIs with Microsoft Graph - Todd Baginski"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["sharepoint-driveitems", "graph-driveitems", "chunked-upload", "permissions-inspection"]
---

# copilot-graph-sharepoint-driveitems
> Based on **SharePoint & OneDrive REST APIs with Microsoft Graph - Todd Baginski** (Cluster 4)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Upload Session Invariant: Files larger than 4MB MUST use uploadSession chunked upload.**
2. **DriveItem Hierarchy: Navigate files via `/drives/{drive-id}/root:/{path}`.**
3. **MANDATORY checksum validation using quickXorHash or sha256.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-graph-sharepoint-driveitems.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-graph-sharepoint-driveitems.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Automate file lifecycle in SharePoint and OneDrive via Graph DriveItem APIs, utilizing chunked upload sessions for large documents.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-graph-sharepoint-driveitems.**
- **Unmonitored runtime execution without telemetry in copilot-graph-sharepoint-driveitems.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "sharepoint-driveitems"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
