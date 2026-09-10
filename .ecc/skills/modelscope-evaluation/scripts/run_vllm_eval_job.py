#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Run local vLLM-style evaluation wrappers."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent


def main() -> None:
    parser = argparse.ArgumentParser(description="Local vLLM evaluation launcher")
    parser.add_argument("--framework", choices=["inspect", "lighteval"], default="inspect")
    parser.add_argument("--model", required=True)
    parser.add_argument("--task", required=True)
    parser.add_argument("--limit", type=int, default=100)
    args = parser.parse_args()

    script = SCRIPT_DIR / ("inspect_vllm_uv.py" if args.framework == "inspect" else "lighteval_vllm_uv.py")
    cmd = ["uv", "run", str(script), "--model", args.model, "--task", args.task, "--limit", str(args.limit)]
    subprocess.run(cmd, check=True)


if __name__ == "__main__":
    main()
