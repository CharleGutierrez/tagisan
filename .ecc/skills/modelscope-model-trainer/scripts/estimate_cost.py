#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Simple training cost estimator for ms-swift runs."""

from __future__ import annotations

import argparse

GPU_HOURLY_USD = {
    "t4": 0.35,
    "a10": 1.2,
    "a100": 3.8,
    "h100": 7.5,
}


def estimate_hours(tokens_billion: float, throughput_tokens_per_sec: float, gpus: int) -> float:
    total_tokens = tokens_billion * 1_000_000_000
    return total_tokens / max(1.0, throughput_tokens_per_sec * gpus) / 3600


def main() -> None:
    parser = argparse.ArgumentParser(description="Estimate runtime and cost for language-model fine-tuning")
    parser.add_argument("--tokens-b", type=float, required=True, help="Training tokens in billions")
    parser.add_argument("--throughput", type=float, required=True, help="Per-GPU tokens/sec")
    parser.add_argument("--gpus", type=int, default=1, help="GPU count")
    parser.add_argument("--gpu", choices=sorted(GPU_HOURLY_USD.keys()), default="a10", help="GPU class")
    args = parser.parse_args()

    hours = estimate_hours(args.tokens_b, args.throughput, args.gpus)
    cost = hours * args.gpus * GPU_HOURLY_USD[args.gpu]

    print(f"Estimated wall-clock hours: {hours:.2f}")
    print(f"Estimated cost (USD): {cost:.2f}")


if __name__ == "__main__":
    main()
