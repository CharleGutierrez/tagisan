---
name: copilot-metacognitive-self-reflection
description: "Self-critique, verification rubrics, and consistency sampling before tool execution."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Automated Reprompting and Self-Refinement Loops - Noah Shinn"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["metacognitive-reflection", "self-critique-loops", "verification-rubrics", "consistency-sampling"]
---

# copilot-metacognitive-self-reflection
> Based on **Automated Reprompting and Self-Refinement Loops - Noah Shinn** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Reflection Step: Prompt model to critique its own candidate solution against a 5-point quality checklist.**
2. **Correction Loop: If critique identifies violations, regenerate the response addressing specific critique findings.**
3. **ALWAYS perform self-reflection before executing destructive database or API actions.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-metacognitive-self-reflection.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-metacognitive-self-reflection actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Equip agents with metacognitive self-reflection passes, catching factual errors and formatting bugs prior to emission.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-metacognitive-self-reflection.**
- **Unmonitored runtime execution without telemetry in copilot-metacognitive-self-reflection.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "metacognitive-reflection"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
