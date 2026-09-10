# Hardware Guide

This guide helps choose runtime profiles for workflow scripts in this skill.

## Baseline Profiles

- `cpu-small`: metadata extraction, lightweight preprocessing.
- `cpu-large`: parquet scans, large joins, report generation.
- `gpu-1x`: single-model inference and embedding generation.
- `gpu-4x`: high-throughput generation or multi-shard inference.

## Quick Sizing Rules

- Prefer CPU for purely tabular transforms.
- Use a single GPU for models up to ~8B when latency is acceptable.
- Use tensor-parallel or multi-GPU for larger models and strict throughput goals.
- Measure token throughput in the first 200 requests before scaling.

## Runtime Examples

```bash
# CPU workflow
uv run scripts/finepdfs-stats.py --source-dataset your-org/raw-docs --output-repo your-org/doc-stats

# Single GPU workflow
CUDA_VISIBLE_DEVICES=0 uv run scripts/generate-responses.py \
  --model Qwen/Qwen2.5-7B-Instruct \
  --source-dataset your-org/prompts \
  --output-dataset your-org/generated
```

```bash
# Multi-GPU workflow
torchrun --nproc_per_node=4 scripts/generate-responses.py \
  --model Qwen/Qwen2.5-14B-Instruct \
  --source-dataset your-org/prompts \
  --output-dataset your-org/generated
```

## Cost Controls

- Cap max samples during dry runs.
- Save intermediate results every N records.
- Stop early on repeated model/runtime failures.
- Use deterministic seeds for retries and diffing.
