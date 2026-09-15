---
name: agentic-huyen-ai-engineering
description: "Production AI systems, prompt chaining, structured outputs (JSON schema), latency/cost trade-offs, evaluation cascades, and enterprise agent deployment."
triggers: ["huyen", "chip-huyen", "ai-engineering", "structured-outputs", "evaluation-cascades", "prompt-chaining", "foundation-models"]
---

# agentic-huyen-ai-engineering
> Based on **AI Engineering: Building Applications with Foundation Models - Chip Huyen**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Structured Output Enforcement: Guarantee valid JSON/Pydantic schemas via grammar-constrained sampling or JSON mode.**
2. **Cost-Latency-Quality Frontier: Route queries across small local models (fast/cheap) and frontier cloud models based on task complexity.**
3. **Evaluation Cascades: Multi-stage evaluation combining regex unit tests, AST parsers, and LLM-as-a-judge rubrics.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement strict JSON schema contracts for all agent tool calls. Route routine code tasks to local LLMs and escalate architectural refactoring to frontier models.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Allowing unconstrained free-text output when deterministic JSON schemas are required by downstream tools.**
- **Using expensive frontier models for trivial regex extractions or boilerplate formatting.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "huyen"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
