---
name: copilot-groundedness-hallucination-detector
description: "Detecting hallucinations with synthetic contrastive verification and model-graded critique."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "LLM-as-a-Judge: Automated Evaluation Frameworks - Lianmin Zheng et al."
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["hallucination-detector", "groundedness-eval", "synthetic-verification", "model-graded-critique"]
---

# copilot-groundedness-hallucination-detector
> Based on **LLM-as-a-Judge: Automated Evaluation Frameworks - Lianmin Zheng et al.** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Claim Extraction: Deconstruct candidate answer into individual atomic factual propositions.**
2. **Entailment Verification: Verify that each atomic claim is logically entailed by the retrieved source context.**
3. **STRICT_REJECT answers containing ungrounded claims.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-groundedness-hallucination-detector.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-groundedness-hallucination-detector.**
6. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-groundedness-hallucination-detector actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Deploy automated hallucination detection gates that verify every generated claim against source grounding documents.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-groundedness-hallucination-detector.**
- **Unmonitored runtime execution without telemetry in copilot-groundedness-hallucination-detector.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "hallucination-detector"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
