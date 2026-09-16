---
name: copilot-power-automate-cloud-flows-ai
description: "Automated and instant cloud flows triggered by Copilot, AI Prompts action, and JSON parsing."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Automating Business Processes with Power Automate and AI Builder - Aaron Murchie"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-automate-copilot", "cloud-flows-ai", "ai-prompts-action", "power-automate-json"]
---

# copilot-power-automate-cloud-flows-ai
> Based on **Automating Business Processes with Power Automate and AI Builder - Aaron Murchie** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Flow Trigger Contract: Trigger flows via Copilot Studio using strongly typed input parameters.**
2. **Output Schema: Always return a structured JSON response schema to Copilot within 60 seconds.**
3. **MANDATORY error handling using Scope blocks with run-after conditions.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-power-automate-cloud-flows-ai.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-power-automate-cloud-flows-ai.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Build automated cloud flows in Power Automate that serve as high-reliability backend tools for Microsoft Copilot.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-power-automate-cloud-flows-ai.**
- **Unmonitored runtime execution without telemetry in copilot-power-automate-cloud-flows-ai.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-automate-copilot"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
