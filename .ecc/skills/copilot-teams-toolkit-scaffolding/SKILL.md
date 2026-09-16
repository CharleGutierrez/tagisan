---
name: copilot-teams-toolkit-scaffolding
description: "Teams Toolkit CLI and VS Code architecture, lifecycle hooks (teamsapp.yml), provisioning, and environment orchestration."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Building Microsoft 365 Solutions with Teams Toolkit - John Miller"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-toolkit", "teams-toolkit-cli", "teamsapp-yml", "teams-scaffolding"]
---

# copilot-teams-toolkit-scaffolding
> Based on **Building Microsoft 365 Solutions with Teams Toolkit - John Miller** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Lifecycle Phases: `provision`, `deploy`, `publish` lifecycle stages defined in `teamsapp.yml`.**
2. **Environment Isolation: Independent parameterization for `dev`, `test`, `prod` via `.env.{env}` files.**
3. **Deterministic Packaging: Manifest templates (`manifest.json`) interpolate environment variables at build time.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-teams-toolkit-scaffolding.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-teams-toolkit-scaffolding.**
6. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-toolkit-scaffolding actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Utilize Teams Toolkit CLI (`teamsapp`) for automated CI/CD pipeline provisioning, validation, and package generation.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Checking plain-text client secrets or bot passwords into source control.**
- **Editing generated `manifest.json` in `build/` directly instead of the source template in `appPackage/`.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-toolkit"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
