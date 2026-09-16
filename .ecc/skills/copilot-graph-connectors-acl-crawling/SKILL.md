---
name: copilot-graph-connectors-acl-crawling
description: "External item ACL mapping, user identity resolution, and security trimming in Semantic Index."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Building Microsoft Graph Connectors - Microsoft Press"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["graph-connectors-acl", "acl-crawling", "identity-resolution", "security-trimming"]
---

# copilot-graph-connectors-acl-crawling
> Based on **Building Microsoft Graph Connectors - Microsoft Press** (Cluster 4)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Security Trimming Invariant: External items MUST inherit exact access control entries (Acl) mapped to Entra ID users/groups.**
2. **Deny Rules: Explicit deny ACLs override grant ACLs unconditionally.**
3. **MANDATORY verification that unauthenticated users cannot access indexed external documents.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-graph-connectors-acl-crawling.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-graph-connectors-acl-crawling.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Crawl and synchronize Access Control Lists along with content items to guarantee enterprise security trimming in Copilot search results.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-graph-connectors-acl-crawling.**
- **Unmonitored runtime execution without telemetry in copilot-graph-connectors-acl-crawling.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "graph-connectors-acl"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
