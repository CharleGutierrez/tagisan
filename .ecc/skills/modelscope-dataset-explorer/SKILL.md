---
name: modelscope-dataset-explorer
description: Explore ModelScope datasets from plain-language inspection requests using metadata
  lookup, selective download, and local profiling.
...
---
# ModelScope Dataset Explorer

Use this skill when the user wants to understand an existing dataset before training, cleaning, or publication.

## Request Style

- Accept requests such as:
  - `Find a Chinese instruction dataset for SFT.`
  - `检查这个数据集的列、切分和样本质量。`
- Start from metadata and only download files when inspection really needs local access.

## Workflow

1. Resolve the target dataset from the request.
2. Inspect metadata first.
3. Download only the smallest useful subset.
4. Run local profiling or SQL queries.
5. Return concrete findings, not generic praise.

## What To Report

- dataset id
- split layout
- schema summary
- file formats
- representative samples when safe
- concrete risks such as duplicates, imbalance, missing fields, or license ambiguity

## Tools

- ModelScope metadata APIs or CLI download
- DuckDB for parquet/csv/jsonl profiling

## Guardrails

- Never dump sensitive rows into logs.
- Never download the whole dataset when metadata or a filtered subset is enough.
- Never recommend a dataset without mentioning real caveats.
