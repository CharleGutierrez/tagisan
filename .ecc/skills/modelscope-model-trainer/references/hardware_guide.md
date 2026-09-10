# Hardware Guide

- 1x consumer GPU: pilot runs, low-rank adapters, small datasets.
- 1x A10/A100 class GPU: standard 7B fine-tuning.
- 4x GPUs: larger context windows, higher throughput, GRPO variants.

## Practical Advice

- Validate memory with a 200-step smoke run.
- Scale sequence length before scaling batch size.
- Use gradient accumulation to fit memory safely.
