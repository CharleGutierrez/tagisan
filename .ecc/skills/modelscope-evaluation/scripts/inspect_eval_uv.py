#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Deterministic local evaluation stub for inspect-style tasks."""

from __future__ import annotations

import argparse
import hashlib
import json


def deterministic_score(model: str, task: str) -> float:
    seed = hashlib.sha256(f"{model}:{task}".encode("utf-8")).hexdigest()
    return round((int(seed[:8], 16) % 4000) / 100.0 + 50, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--task", required=True)
    parser.add_argument("--limit", type=int, default=100)
    args = parser.parse_args()

    payload = {
        "framework": "inspect-local",
        "model": args.model,
        "task": args.task,
        "limit": args.limit,
        "score": deterministic_score(args.model, args.task),
    }
    print(json.dumps(payload, indent=2))


if __name__ == "__main__":
    main()
