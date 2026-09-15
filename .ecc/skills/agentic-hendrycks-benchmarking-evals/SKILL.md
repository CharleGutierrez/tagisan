---
name: agentic-hendrycks-benchmarking-evals
description: "Benchmark design, multi-choice evaluation rubrics, calibration curves, normalized scoring, and contamination/leakage detection."
triggers: ["hendrycks", "mmlu", "llm-benchmarking", "evaluation-rubrics", "calibration-curves", "contamination-detection"]
---

# agentic-hendrycks-benchmarking-evals
> Based on **Measuring Massive Multitask Language Understanding (MMLU) and Benchmarking - Dan Hendrycks et al.**

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Normalized Scoring: Evaluating agents against standardized multidimensional benchmark suites across zero-shot and few-shot splits.**
2. **Model Calibration: Confidence scores must reflect true empirical accuracy: E_{(X, Y)}[|P(Y=y | P_pred=p) - p|] -> 0.**
3. **Data Leakage Auditing: Ensuring benchmark evaluation tasks are not present in the agent's pre-training or fine-tuning datasets.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Establish rigorous internal evals for code generation agents. Track pass@1 and pass@k across diverse coding tasks to detect regression before deploying.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Evaluating agents only on synthetic tasks identical to prompt examples.**
- **Relying on subjective human vibes without quantitative automated benchmark metrics.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "hendrycks"

# Execute automated agentic engineering audit
cargo test --test agentic_skills_brutal_tests
```
