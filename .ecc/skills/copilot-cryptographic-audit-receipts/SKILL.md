---
name: copilot-cryptographic-audit-receipts
description: "Generating SHA-256 tamper-evident cryptographic audit logs for every Copilot agent invocation."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Observability and Audit Logging for Autonomous Enterprise Agents - Charity Majors"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["cryptographic-audit-receipts", "tamper-evident-logs", "sha256-audit-chain", "immutable-audit-records"]
---

# copilot-cryptographic-audit-receipts
> Based on **Observability and Audit Logging for Autonomous Enterprise Agents - Charity Majors** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Audit Hash Chain: Every invocation record contains `Hash = SHA256(PrevHash + Timestamp + QueryHash + ActionHash)`.**
2. **Immutability: Write audit records to immutable Azure Blob Storage (WORM - Write Once, Read Many).**
3. **MANDATORY inclusion of user Entra ID object ID, tenant ID, and IP address in audit payload.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-cryptographic-audit-receipts.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-cryptographic-audit-receipts.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Generate cryptographic hash-chained audit receipts for all agent actions to provide tamper-evident compliance trails.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-cryptographic-audit-receipts.**
- **Unmonitored runtime execution without telemetry in copilot-cryptographic-audit-receipts.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cryptographic-audit-receipts"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
