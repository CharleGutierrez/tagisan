# Quality Gates and Alerts

This skill uses run status + summary fields to represent alert outcomes in a portable way.

## Failure Gate Pattern

```bash
uv run scripts/run_tracker.py complete-run \
  --run-id <run-id> \
  --status failed \
  --summary-json '{"gate":"nan_loss_detected","threshold":"loss is NaN"}'
```

## Warning-Only Pattern

Keep run status as `succeeded` but attach warning metadata:

```bash
uv run scripts/run_tracker.py complete-run \
  --run-id <run-id> \
  --status succeeded \
  --summary-json '{"warning":"eval_accuracy_below_target","target":0.75,"actual":0.71}'
```

## Suggested Gates

- NaN/Inf metric detection
- Throughput below SLA
- Validation metric regression vs baseline
- Missing expected artifact outputs
