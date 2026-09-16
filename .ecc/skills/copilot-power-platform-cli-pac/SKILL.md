---
name: copilot-power-platform-cli-pac
description: "Power Platform CLI (pac) command automation, solution pack/unpack, and pipeline deployment."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Application Lifecycle Management for Copilot Studio - Microsoft Power CAT"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["power-platform-cli", "pac-cli-automation", "solution-unpack", "pac-admin-pipeline"]
---

# copilot-power-platform-cli-pac
> Based on **Application Lifecycle Management for Copilot Studio - Microsoft Power CAT** (Cluster 7)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **CLI Automation: Automate ALM via `pac solution pack`, `pac solution unpack`, `pac solution export`.**
2. **Source Control Friendly: Deconstruct solutions into component XML/YAML files suitable for Git diffs.**
3. **MANDATORY automated solution check (`pac solution check`) before release.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-power-platform-cli-pac.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-power-platform-cli-pac.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Automate Power Platform ALM and solution deployment using `pac` commands in continuous integration pipelines.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-power-platform-cli-pac.**
- **Unmonitored runtime execution without telemetry in copilot-power-platform-cli-pac.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "power-platform-cli"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
