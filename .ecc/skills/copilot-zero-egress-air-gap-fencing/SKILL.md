---
name: copilot-zero-egress-air-gap-fencing
description: "Configuring network boundaries, private endpoints, and egress locks for enterprise data protection."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Zero Trust Security for Enterprise AI Systems - Jason Garbis & Jerry Chapman"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["zero-egress-fencing", "air-gap-enterprise", "private-endpoints", "network-security-perimeter"]
---

# copilot-zero-egress-air-gap-fencing
> Based on **Zero Trust Security for Enterprise AI Systems - Jason Garbis & Jerry Chapman** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Network Perimeter: Route all Copilot backend communication through Azure Virtual Network Private Endpoints.**
2. **Egress Lock: Prohibit outbound public internet traffic from inference and data processing subnets.**
3. **MANDATORY TLS 1.3 encryption with client certificate pinning on internal service links.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-zero-egress-air-gap-fencing.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-zero-egress-air-gap-fencing.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Architect zero-egress network perimeters around enterprise Copilot extensions to guarantee data containment.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-zero-egress-air-gap-fencing.**
- **Unmonitored runtime execution without telemetry in copilot-zero-egress-air-gap-fencing.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "zero-egress-fencing"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
