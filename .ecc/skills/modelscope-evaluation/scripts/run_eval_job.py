#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Run inspect-style evaluation locally."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

SCRIPT = Path(__file__).with_name("inspect_eval_uv.py")


def main() -> None:
    parser = argparse.ArgumentParser(description="Local inspect-style evaluation runner")
    parser.add_argument("--model", required=True)
    parser.add_argument("--task", required=True)
    parser.add_argument("--limit", type=int, default=100)
    args = parser.parse_args()

    cmd = ["uv", "run", str(SCRIPT), "--model", args.model, "--task", args.task, "--limit", str(args.limit)]
    subprocess.run(cmd, check=True)


if __name__ == "__main__":
    main()
