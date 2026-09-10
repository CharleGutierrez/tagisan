# Logging Metrics

Use `run_tracker.py` as the canonical writer for run metadata and metrics.

## Create a Run Manifest

```bash
uv run scripts/run_tracker.py create-run \
  --model-id Qwen/Qwen2.5-7B-Instruct \
  --dataset-id your-org/train-data \
  --task sft \
  --params-json '{"lr":2e-5,"batch_size":8}'
```

## Append Metrics

```bash
uv run scripts/run_tracker.py log-metric --run-id <run-id> --name train_loss --value 1.42 --step 1
uv run scripts/run_tracker.py log-metric --run-id <run-id> --name train_loss --value 1.11 --step 10
uv run scripts/run_tracker.py log-metric --run-id <run-id> --name eval_accuracy --value 0.73 --step 200 --split validation
```

## Register Artifacts

```bash
uv run scripts/run_tracker.py add-artifact \
  --run-id <run-id> \
  --path ./outputs/checkpoint-200 \
  --kind checkpoint \
  --label "checkpoint-200"
```

## Recommendations

- Log metrics at deterministic intervals.
- Keep step indexing monotonic.
- Use one consistent metric naming policy across training and evaluation scripts.
