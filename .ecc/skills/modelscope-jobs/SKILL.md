---
name: modelscope-jobs
description: Execute plain-language batch-job requests as reproducible local or PAI DLC workflows
  and publish the outputs to ModelScope.
...
---
# ModelScope Jobs

Use this skill when the user wants a batch workflow such as large-scale inference, data generation, statistics, or report production.

## Request Style

- Accept natural requests like:
  - `Generate answers for this prompt dataset with Qwen2.5 and publish the result.`
  - `统计这个 PDF 数据集的页面数并输出 markdown 报告。`
- Infer the lightest viable runtime:
  - local for small jobs
  - PAI DLC for long or heavy jobs

## Primary Scripts

- `scripts/generate-responses.py`
- `scripts/cot-self-instruct.py`
- `scripts/finepdfs-stats.py`
- `scripts/submit_pai_dlc.py`

## Workflow

1. Translate the user request into one of the existing scripts or a short `USER_COMMAND`.
2. Reuse local files when they already exist.
3. If the run must go remote, prepare `configs/remote.auto.env`.
4. If required PAI values are missing, create `configs/pai.required.env`, ask the user to fill it, and wait for `continue`.
5. Upload local assets when the remote job needs them.
6. Save outputs under stable local paths and publish only when credentials allow.

## Credential Gate

For real PAI DLC submission, pause until these exist:

- `ALIBABA_CLOUD_ACCESS_KEY_ID`
- `ALIBABA_CLOUD_ACCESS_KEY_SECRET`
- `PAI_REGION`
- `PAI_WORKSPACE_ID`

Create `configs/pai.required.env` with those four placeholders when needed.

## Output Rules

- Put machine-readable results under `data/` or another explicit output folder.
- Put human-readable summaries under `reports/`.
- Publish to ModelScope only when the target repo and token are available.

## References

- `references/hardware_guide.md`
- `references/hub_saving.md`
- `references/troubleshooting.md`
- `references/token_usage.md`

## Guardrails

- Never force the user to describe the job as a rigid prompt schema.
- Never assume local paths exist remotely.
- Never hide whether the job ran locally or on PAI DLC.
- Never publish private raw data by default.
