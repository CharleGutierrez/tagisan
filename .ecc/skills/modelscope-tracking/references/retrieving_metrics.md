# Retrieving and Reporting

## List Runs

```bash
uv run scripts/run_tracker.py list-runs
uv run scripts/run_tracker.py list-runs --format json
```

## Export Metrics to CSV

```bash
uv run scripts/run_tracker.py export-metrics-csv \
  --run-id <run-id> \
  --output .modelscope-tracking/reports/<run-id>.csv
```

## Build Project Summaries

```bash
uv run scripts/run_tracker.py summarize --format markdown --output .modelscope-tracking/reports/summary.md
uv run scripts/run_tracker.py summarize --format json --output .modelscope-tracking/reports/summary.json
```

## Validate Workspace

```bash
uv run scripts/run_tracker.py validate
```

Use JSON outputs for automation and markdown outputs for human review in PRs or model cards.
