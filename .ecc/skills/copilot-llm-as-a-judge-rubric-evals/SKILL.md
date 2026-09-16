---
name: copilot-llm-as-a-judge-rubric-evals
description: "Formulating multi-dimensional rubric prompts for LLM-as-a-judge evaluation of agent performance."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "LLM-as-a-Judge: Automated Evaluation Frameworks - Lianmin Zheng et al."
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["llm-as-a-judge", "rubric-evals", "automated-grading-prompts", "pairwise-model-comparison"]
---

# copilot-llm-as-a-judge-rubric-evals
> Based on **LLM-as-a-Judge: Automated Evaluation Frameworks - Lianmin Zheng et al.** (Cluster 14)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Rubric Dimensions: Evaluate on Clarity, Completeness, Groundedness, Conciseness, and Actionability.**
2. **Scoring Scale: 1-5 integer scale with explicit descriptions for each rating tier.**
3. **ALWAYS calibrate judge model against a human-validated golden baseline dataset.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-llm-as-a-judge-rubric-evals.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-llm-as-a-judge-rubric-evals actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Establish automated LLM-as-a-judge evaluation pipelines with structured rubrics to assess agent answer quality at scale.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-llm-as-a-judge-rubric-evals.**
- **Unmonitored runtime execution without telemetry in copilot-llm-as-a-judge-rubric-evals.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "llm-as-a-judge"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
