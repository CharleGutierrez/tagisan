#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Estimate training cost for vision workloads."""

from __future__ import annotations

import argparse

GPU_USD = {"t4": 0.35, "a10": 1.2, "a100": 3.8}


def main() -> None:
    parser = argparse.ArgumentParser(description="Estimate vision training cost")
    parser.add_argument("--hours", type=float, required=True)
    parser.add_argument("--gpus", type=int, default=1)
    parser.add_argument("--gpu", choices=sorted(GPU_USD.keys()), default="a10")
    args = parser.parse_args()

    cost = args.hours * args.gpus * GPU_USD[args.gpu]
    print(f"Estimated cost (USD): {cost:.2f}")


if __name__ == "__main__":
    main()
