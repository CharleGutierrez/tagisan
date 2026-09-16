---
name: copilot-declarative-agent-publishing-admin
description: "Tenant sideloading, Integrated Apps publishing, and M365 Admin Center approval workflows."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft 365 Administration Inside Out - Ed Fisher & Darryl van der Peijl"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["publishing-admin", "integrated-apps-publishing", "m365-admin-approval", "tenant-sideloading"]
---

# copilot-declarative-agent-publishing-admin
> Based on **Microsoft 365 Administration Inside Out - Ed Fisher & Darryl van der Peijl** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Publishing Channels: 1) Sideloading (developer testing), 2) Organization App Catalog (tenant-wide), 3) Commercial Marketplace.**
2. **Admin Consent Workflow: Admin reviews requested Graph permissions, sensitivity labels, and developer info.**
3. **ALWAYS secure tenant admin consent prior to enterprise-wide rollout.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-declarative-agent-publishing-admin.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-declarative-agent-publishing-admin actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Prepare administrative governance documentation outlining data access scopes, external APIs, and business justification before submitting for tenant review.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Directly sideloading unverified developer builds into production executive accounts.**
- **Requesting broad `Directory.ReadWrite.All` scopes when simple `User.Read` suffices.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "publishing-admin"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
