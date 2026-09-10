---
name: modelscope-evaluation
description: Turn plain-language evaluation requests into benchmark extraction, remote evaluation
  jobs, and publishable result artifacts.
...
---
# ModelScope Evaluation

Use this skill when the user wants to extract, normalize, run, or publish model evaluation results.

## Request Style

- Accept natural requests such as:
  - `Run an MMLU pilot for this model.`
  - `把 README 里的评测表抽出来，整理成结构化 JSON。`
- First decide whether the task is:
  - metadata extraction
  - score normalization
  - a real evaluation run

## Primary Scripts

- `scripts/evaluation_manager.py`
- `scripts/run_eval_job.py`
- `scripts/run_vllm_eval_job.py`
- `scripts/inspect_eval_uv.py`
- `scripts/inspect_vllm_uv.py`
- `scripts/lighteval_vllm_uv.py`
- `scripts/submit_pai_dlc.py`

## Workflow

1. Inspect existing README tables, JSON, YAML, and logs before generating anything new.
2. Normalize benchmark names and units.
3. Use local extraction for lightweight tasks.
4. Use PAI DLC only for real model evaluation or long-running inference.
5. When remote submission is needed, create `configs/pai.required.env` if required PAI values are missing and wait for `continue`.
6. Save machine-readable outputs under `eval/` or another explicit directory.

## Credential Gate

Pause remote execution until these exist:

- `ALIBABA_CLOUD_ACCESS_KEY_ID`
- `ALIBABA_CLOUD_ACCESS_KEY_SECRET`
- `PAI_REGION`
- `PAI_WORKSPACE_ID`

## Output Rules

Always preserve provenance:

- source file or evaluator
- model identifier
- task name
- timestamp
- normalization decisions

## Examples And References

- `examples/USAGE_EXAMPLES.md`
- `examples/example_readme_tables.md`
- `examples/metric_mapping.json`

## Guardrails

- Never fabricate benchmark values.
- Never merge incompatible units into one score field.
- Never overwrite existing scores without an explicit merge rule.
- Prefer dry-run style extraction before mutating repository content.
