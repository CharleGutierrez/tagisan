---
name: copilot-azure-openai-private-link-network
description: "Securing Azure OpenAI traffic via Virtual Network Private Endpoints and disabled public access."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Architecting Enterprise Solutions with Azure OpenAI Service - Sarah Kaiser"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["azure-openai-private-link", "vnet-private-endpoints", "disabled-public-access", "secure-ai-networking"]
---

# copilot-azure-openai-private-link-network
> Based on **Architecting Enterprise Solutions with Azure OpenAI Service - Sarah Kaiser** (Cluster 12)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Private Link: Map Azure OpenAI resource to private IP addresses within enterprise virtual network.**
2. **Public Access Invariant: Disable `publicNetworkAccess` on all production cognitive services.**
3. **MANDATORY private DNS zone (`privatelink.openai.azure.com`) resolution within internal VNets.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-azure-openai-private-link-network.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-azure-openai-private-link-network.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Secure Azure OpenAI communication over private virtual network endpoints, preventing data exposure to public internet.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-azure-openai-private-link-network.**
- **Unmonitored runtime execution without telemetry in copilot-azure-openai-private-link-network.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "azure-openai-private-link"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
