---
name: copilot-studio-alm-solution-lifecycle
description: "Power Platform Solutions, environment variables, ALM pipelines, and automated export/import."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Application Lifecycle Management for Copilot Studio - Microsoft Power CAT"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-studio-alm", "solution-lifecycle", "power-platform-alm", "managed-solutions"]
---

# copilot-studio-alm-solution-lifecycle
> Based on **Application Lifecycle Management for Copilot Studio - Microsoft Power CAT** (Cluster 3)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Managed Solutions: Packaging bots, topics, flows, and connection references into immutable Managed Solutions for production.**
2. **Environment Variables: Abstracting endpoint URLs, client IDs, and tenant configs across Dev, Test, Prod.**
3. **MANDATORY source code versioning of unpacked solution files using Power Platform CLI (`pac`).**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-studio-alm-solution-lifecycle.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-studio-alm-solution-lifecycle.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce strict CI/CD pipelines in Azure DevOps or GitHub Actions: unpack solution -> commit to Git -> run test suite -> build managed solution -> deploy to Prod.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Directly editing topics and flows in production environments ('hot patching').**
- **Hardcoding developer tenant connection IDs inside solution components.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-studio-alm"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
