---
name: copilot-azure-ai-content-safety-api
description: "Integrating Azure AI Content Safety for real-time text analysis of hate, violence, sexual, self-harm."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Azure AI Content Safety and Risk Mitigation - Microsoft AI Team"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["azure-content-safety", "content-moderation-api", "harm-severity-levels", "prompt-shields-scanning"]
---

# copilot-azure-ai-content-safety-api
> Based on **Azure AI Content Safety and Risk Mitigation - Microsoft AI Team** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Severity Classification: Quantify risk across 4 categories (Hate, Violence, Sexual, Self-Harm) on 0-7 severity scale.**
2. **Prompt Shields: Scan inputs for user prompt injection attacks and document context attacks.**
3. **ALWAYS block inputs/outputs exceeding tenant risk thresholds.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-azure-ai-content-safety-api.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-azure-ai-content-safety-api actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Integrate Azure AI Content Safety APIs into agent middleware to enforce real-time enterprise moderation and safety.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-azure-ai-content-safety-api.**
- **Unmonitored runtime execution without telemetry in copilot-azure-ai-content-safety-api.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "azure-content-safety"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
