---
name: copilot-purview-sensitivity-labeling
description: "Discovering, reading, and applying Microsoft Purview sensitivity labels to generated artifacts."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft Purview: Data Governance and Information Protection - Santhosh Balasubramanian"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["purview-sensitivity-labeling", "sensitivity-labels", "information-protection", "purview-governance"]
---

# copilot-purview-sensitivity-labeling
> Based on **Microsoft Purview: Data Governance and Information Protection - Santhosh Balasubramanian** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Label Inheritance Invariant: Generated documents MUST inherit the highest sensitivity label of all input sources.**
2. **MIP SDK Integration: Apply encryption and visual watermarks via Microsoft Information Protection (MIP) SDK.**
3. **NEVER downgrade sensitivity labels without explicit administrative authorization.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-purview-sensitivity-labeling.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-purview-sensitivity-labeling actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Integrate Microsoft Purview sensitivity labeling into Copilot artifact generation, enforcing label inheritance rules.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-purview-sensitivity-labeling.**
- **Unmonitored runtime execution without telemetry in copilot-purview-sensitivity-labeling.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "purview-sensitivity-labeling"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
