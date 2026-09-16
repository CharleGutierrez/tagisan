---
name: copilot-purview-dlp-data-fencing
description: "Enforcing DLP policies against sensitive data (PII, PCI, HIPAA) in Copilot prompts and completions."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Purview: Data Governance and Information Protection - Santhosh Balasubramanian"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["purview-dlp", "data-loss-prevention", "pii-pci-fencing", "dlp-policy-enforcement"]
---

# copilot-purview-dlp-data-fencing
> Based on **Microsoft Purview: Data Governance and Information Protection - Santhosh Balasubramanian** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **DLP Detection: Scan prompts and completions for Sensitive Information Types (SITs) in real time.**
2. **Blocking Rule: Automatically block or redact credit card numbers, social security numbers, and health records.**
3. **MANDATORY incident generation in Microsoft Purview Compliance Portal upon DLP violation.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-purview-dlp-data-fencing.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-purview-dlp-data-fencing.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Enforce Microsoft Purview DLP rules across all agent conversation streams to prevent unauthorized data exfiltration.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-purview-dlp-data-fencing.**
- **Unmonitored runtime execution without telemetry in copilot-purview-dlp-data-fencing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "purview-dlp"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
