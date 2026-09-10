#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Deterministic local lighteval-style evaluation stub."""

from __future__ import annotations

import argparse
import hashlib
import json


def score(model: str, task: str) -> float:
    key = hashlib.sha1(f"lighteval:{model}:{task}".encode("utf-8")).hexdigest()
    return round((int(key[:6], 16) % 3500) / 100.0 + 52, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--task", required=True)
    parser.add_argument("--limit", type=int, default=100)
    args = parser.parse_args()
    print(json.dumps({"framework": "lighteval-local", "model": args.model, "task": args.task, "score": score(args.model, args.task)}, indent=2))


if __name__ == "__main__":
    main()
