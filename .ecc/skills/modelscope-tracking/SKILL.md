---
name: modelscope-tracking
description: Add local, file-based experiment tracking to ModelScope workflows from plain-language
  requests.
...
---
# ModelScope Tracking

Use this skill when the user wants experiment tracking that stays local, transparent, and easy to publish later.

## Request Style

- Accept requests such as:
  - `Track this SFT run.`
  - `把最近几次实验整理成可比较的报告。`
- If tracking is not initialized, initialize it automatically in the current project.

## Primary Script

- `scripts/run_tracker.py`

## Workflow

1. Initialize `.modelscope-tracking/` when missing.
2. Create or reuse a run id.
3. Log metrics and artifacts as files, not opaque state.
4. Summarize runs into Markdown, JSON, or CSV.
5. Publish reports later with ordinary ModelScope upload workflows.

## Directory Layout

```text
.modelscope-tracking/
  project.json
  runs/
  artifacts/
  reports/
```

## References

- `references/logging_metrics.md`
- `references/retrieving_metrics.md`
- `references/alerts.md`

## Guardrails

- Never store secrets in tracking files.
- Keep metric names stable across runs.
- Prefer append-only logs over in-place mutation.
