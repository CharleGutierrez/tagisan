---
name: copilot-office-scripts-power-automate-sync
description: "Executing Office Scripts from Power Automate flows, passing dynamic arrays, and receiving return values."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Connecting Office Scripts to Power Automate Cloud Flows - Damien Bird"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["office-scripts-power-automate", "headless-excel-execution", "script-parameter-passing", "excel-cloud-flow"]
---

# copilot-office-scripts-power-automate-sync
> Based on **Connecting Office Scripts to Power Automate Cloud Flows - Damien Bird** (Cluster 8)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Headless Execution: Power Automate executes Office Scripts headlessly on Excel Online servers.**
2. **Typed Interfaces: Define explicit TypeScript interfaces for script input and output parameters.**
3. **MANDATORY timeout consideration: Headless script execution terminates after 120 seconds.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-office-scripts-power-automate-sync.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-office-scripts-power-automate-sync.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Bridge enterprise cloud workflows to Excel data models by executing parameter-driven Office Scripts from Power Automate.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-office-scripts-power-automate-sync.**
- **Unmonitored runtime execution without telemetry in copilot-office-scripts-power-automate-sync.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "office-scripts-power-automate"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
