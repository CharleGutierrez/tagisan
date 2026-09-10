---
name: modelscope-datasets
description: Build, clean, validate, and publish ModelScope datasets from plain-language instructions
  with automatic template selection.
...
---
# ModelScope Datasets

Use this skill when the user wants to create or curate a dataset and describes the requirement in natural language.

## Request Style

- Accept requests like:
  - `Create a chat dataset repo from these JSONL files.`
  - `把这个问答数据整理成标准 QA 数据集并上传到 ModelScope。`
- Infer the nearest supported template instead of demanding a template upfront.
- Default to the closest of:
  - `chat`
  - `classification`
  - `qa`
  - `completion`
  - `tabular`
  - `custom`

## Primary Scripts

- `scripts/dataset_manager.py`
- `scripts/sql_manager.py`

## Workflow

1. Infer dataset type from the user request and existing files.
2. Initialize or reuse the target dataset repo.
3. Normalize rows to the nearest template.
4. Validate before upload.
5. Run SQL-based inspection only after a stable snapshot exists.
6. Publish only the processed dataset, not accidental raw byproducts.

## Defaults

- If split names are missing, prefer `train`, `validation`, `test`.
- If schema is ambiguous, preserve raw columns and choose `custom` instead of forcing the wrong template.
- If metadata is missing, keep placeholders separate from row data.

## Assets

- Templates live in `templates/`.
- Example rows live in `examples/`.

Load only the template or example that matches the current request.

## Authentication

Use `MODELSCOPE_API_TOKEN` or a prior `modelscope login` session for publish steps.

## Guardrails

- Never force the user into a verbose dataset-writing template.
- Never upload private or unlicensed data by default.
- Never mutate rows before validating the intended schema.
- Keep raw and normalized outputs separate.
