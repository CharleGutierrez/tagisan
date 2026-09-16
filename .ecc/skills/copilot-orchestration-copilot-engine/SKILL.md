---
name: copilot-orchestration-copilot-engine
description: "Copilot Orchestrator lifecycle: intent classification, contextual prompt composition, semantic grounding, and completion synthesis."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["copilot-orchestration", "copilot-engine-loop", "intent-classification", "prompt-grounding-synthesis"]
---

# copilot-orchestration-copilot-engine
> Based on **Microsoft 365 Copilot Architecture & Extensibility Guide - Microsoft Press** (Cluster 1)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Orchestrator State Loop: Query -> Intent Extraction -> Semantic Index Query -> Working Context Assembly -> LLM Synthesis -> Post-Processing Guardrails.**
2. **Semantic Grounding Contract: ALWAYS verify that retrieved context chunks match user authorization ACLs before prompt interpolation.**
3. **STRICT_REJECT any synthesis output failing Azure AI Content Safety or prompt injection heuristics.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-orchestration-copilot-engine.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-orchestration-copilot-engine actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Structure Copilot interactions by adhering to the native orchestrator pipeline. Provide clear action schemas with explicit parameter descriptions to maximize intent classification accuracy.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Flooding the orchestrator with ambiguous tool descriptions causing multi-action deadlock.**
- **Assuming single-turn execution when multi-turn clarifying questions are required.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "copilot-orchestration"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
