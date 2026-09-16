---
name: copilot-teams-toolkit-environment-variables
description: "Environment variable management (.env.dev, .env.prod, teamsapp.local.yml) and secrets substitution."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Building Microsoft 365 Solutions with Teams Toolkit - John Miller"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["teams-env-variables", "env-dev-prod", "teamsapp-local-yml", "secrets-substitution"]
---

# copilot-teams-toolkit-environment-variables
> Based on **Building Microsoft 365 Solutions with Teams Toolkit - John Miller** (Cluster 2)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Variable Interpolation: `${{VARIABLE_NAME}}` substitution across manifest and configuration templates.**
2. **Secrets Handling: Sensitive values stored in `.env.{env}.user` and NEVER checked into version control.**
3. **ALWAYS validate required environment variables before initiating build or provision steps.**
4. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-teams-toolkit-environment-variables actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Maintain segregated `.env.dev`, `.env.test`, `.env.prod` files. Automate secret injection from Azure Key Vault in CI/CD environments.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Committing unencrypted `.env.{env}.user` files containing Entra ID client secrets.**
- **Hardcoding URLs and tenant IDs inside `manifest.json` files.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "teams-env-variables"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
