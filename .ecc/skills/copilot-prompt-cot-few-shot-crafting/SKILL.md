---
name: copilot-prompt-cot-few-shot-crafting
description: "Chain-of-Thought (CoT) prompting, structured thought tags, and dynamic few-shot exemplar selection."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "Prompt Engineering for Generative AI - James Phoenix & Mike Taylor"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["cot-prompting", "few-shot-crafting", "thought-tags", "dynamic-exemplars"]
---

# copilot-prompt-cot-few-shot-crafting
> Based on **Prompt Engineering for Generative AI - James Phoenix & Mike Taylor** (Cluster 10)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Reasoning Decomposition: Enforce step-by-step reasoning inside `<thinking>` or `<scratchpad>` blocks.**
2. **Exemplar Quality: Provide 2-3 input/output pairs that illustrate complex boundary conditions and edge cases.**
3. **ALWAYS separate scratchpad thinking from final customer-facing responses.**
4. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-prompt-cot-few-shot-crafting.**
5. **MANDATORY timeout guarantees (30s default) and episodic reflection logging on all copilot-prompt-cot-few-shot-crafting actions.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Craft structured Chain-of-Thought prompts with dynamic few-shot exemplars to maximize reasoning fidelity in Copilot.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-prompt-cot-few-shot-crafting.**
- **Unmonitored runtime execution without telemetry in copilot-prompt-cot-few-shot-crafting.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "cot-prompting"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
