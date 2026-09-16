---
name: copilot-office-add-in-deployment-centralized
description: "Centralized deployment via M365 Admin Center, manifest hosting, and telemetry monitoring."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Publishing and Deploying Enterprise Office Add-ins - Microsoft AppSource"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["office-centralized-deployment", "admin-center-addins", "appsource-submission", "office-manifest-hosting"]
---

# copilot-office-add-in-deployment-centralized
> Based on **Publishing and Deploying Enterprise Office Add-ins - Microsoft AppSource** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Deployment Route: Assign add-ins to specific Entra ID security groups via M365 Admin Center.**
2. **Hosting Invariant: Host web application assets on secure, high-availability HTTPS CDNs with TLS 1.3.**
3. **MANDATORY health check endpoint for monitoring add-in uptime.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-office-add-in-deployment-centralized.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-office-add-in-deployment-centralized.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Manage enterprise-wide Office Add-in rollouts through Centralized Deployment, verifying group assignment and telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-office-add-in-deployment-centralized.**
- **Unmonitored runtime execution without telemetry in copilot-office-add-in-deployment-centralized.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "office-centralized-deployment"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
