# Usage Examples

## Extract From README

```bash
uv run scripts/evaluation_manager.py extract-readme \
  --repo-id your-org/your-model \
  --output ./outputs/extracted.json
```

## Import And Publish

```bash
uv run scripts/evaluation_manager.py import-json \
  --input ./outputs/extracted.json \
  --repo-id your-org/your-model
```

## Local Evaluation Stub

```bash
uv run scripts/run_eval_job.py --model Qwen/Qwen2.5-7B-Instruct --task mmlu --limit 100
```
