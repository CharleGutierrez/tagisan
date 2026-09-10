---
name: modelscope-paper-publisher
description: Turn plain-language publication requests into evidence-backed Markdown reports for
  ModelScope repositories.
...
---
# ModelScope Paper Publisher

Use this skill when the user wants a research-style write-up, release note, or technical summary backed by real project evidence.

## Request Style

- Accept requests like:
  - `Write a short technical report for this fine-tuning run.`
  - `基于这些实验结果生成一篇可上传到仓库的总结。`

## Workflow

1. Gather local evidence first: metrics, README files, configs, outputs.
2. Choose the closest template.
3. Draft locally.
4. Verify claims, citations, and links.
5. Upload only after the content is supported.

## Assets

- Templates: `templates/standard.md`, `templates/arxiv.md`, `templates/ml-report.md`, `templates/modern.md`
- Script: `scripts/paper_manager.py`
- Reference: `references/quick_reference.md`

## Guardrails

- Never fabricate claims, ablations, or benchmark wins.
- Never cite links you did not verify.
- Prefer a shorter accurate report over a longer speculative one.
