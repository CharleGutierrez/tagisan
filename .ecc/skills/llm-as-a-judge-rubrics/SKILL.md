---
name: llm-as-a-judge-rubrics
description: Structured evaluation scoring rubrics, Borda count ranking, and adversarial debate adjudication
---

# LLM-as-a-Judge Evaluation & Adjudication Rubrics

## Evaluation Rubrics
1. **Correctness (Weight 40%)**: Mathematical soundness, constraint satisfaction, compile-time validity, edge-case coverage.
2. **Security & Guardrails (Weight 30%)**: Resistance to injection, privilege escalation, credential exposure, buffer exhaustion.
3. **Maintainability (Weight 15%)**: Modularity, idiomatic naming, documentation density, clear separation of concerns.
4. **Performance (Weight 15%)**: Algorithmic time/space complexity, memory footprint, cache-friendliness.
5. **Consensus Synthesis**: Aggregate individual agent assessments using Positional Borda Count or Supermajority voting.
