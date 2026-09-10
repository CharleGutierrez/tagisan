#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Deterministic local vLLM-style inspect evaluation stub."""

from __future__ import annotations

import argparse
import hashlib
import json


def score(model: str, task: str) -> float:
    key = hashlib.md5(f"inspect-vllm:{model}:{task}".encode("utf-8")).hexdigest()
    return round((int(key[:6], 16) % 3000) / 100.0 + 55, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--task", required=True)
    parser.add_argument("--limit", type=int, default=100)
    args = parser.parse_args()
    print(json.dumps({"framework": "inspect-vllm-local", "model": args.model, "task": args.task, "score": score(args.model, args.task)}, indent=2))


if __name__ == "__main__":
    main()
