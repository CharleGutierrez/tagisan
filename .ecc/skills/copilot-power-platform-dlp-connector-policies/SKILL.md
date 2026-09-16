---
name: copilot-power-platform-dlp-connector-policies
description: "Data Loss Prevention policies, classifying custom connectors (Business vs Non-Business), and tenant isolation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Power Platform Center of Excellence (CoE) Governance - Microsoft IT"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-platform-dlp", "dlp-connector-policies", "business-vs-nonbusiness", "coe-governance"]
---

# copilot-power-platform-dlp-connector-policies
> Based on **Microsoft Power Platform Center of Excellence (CoE) Governance - Microsoft IT** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **DLP Boundary: Connectors partitioned into Business, Non-Business, and Blocked groups.**
2. **Data Exfiltration Invariant: Data CANNOT pass between Business and Non-Business connectors in a single flow.**
3. **ALWAYS verify connector classification before deploying production Copilot flows.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-power-platform-dlp-connector-policies.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-power-platform-dlp-connector-policies actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design and enforce tenant DLP policies that prevent accidental cross-contamination of corporate and personal services.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-power-platform-dlp-connector-policies.**
- **Unmonitored runtime execution without telemetry in copilot-power-platform-dlp-connector-policies.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-platform-dlp"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
