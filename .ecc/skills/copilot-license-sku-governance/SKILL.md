---
name: copilot-license-sku-governance
description: "M365 Copilot license entitlement, SKU feature gating, self-service purchase restrictions, and tenant admin controls."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft 365 Administration Inside Out - Ed Fisher & Darryl van der Peijl"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-license-governance", "sku-feature-gating", "m365-copilot-entitlement", "admin-controls"]
---

# copilot-license-sku-governance
> Based on **Microsoft 365 Administration Inside Out - Ed Fisher & Darryl van der Peijl** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Entitlement Prerequisite: Base prerequisites (M365 E3/E5, Business Standard/Premium) plus Microsoft 365 Copilot Add-on SKU.**
2. **Feature Gating: Enforce programmatically checking user license state before initiating Copilot extension turns.**
3. **ALWAYS fail gracefully with user-friendly remediation steps if license check returns unlicensed.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-license-sku-governance.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-license-sku-governance actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Incorporate automated license and permission pre-flight checks into custom extensions and provisioning scripts.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Assuming all users in a tenant possess Copilot licensing, causing unhandled API 403 Forbidden errors.**
- **Silently failing without alerting users that their account requires a Copilot license.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-license-governance"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
