#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Print an Unsloth-oriented SFT command template."""

from __future__ import annotations

import argparse


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate a memory-efficient SFT command template")
    parser.add_argument("--model", required=True)
    parser.add_argument("--dataset", required=True)
    parser.add_argument("--output", default="./outputs/unsloth-sft")
    args = parser.parse_args()

    cmd = (
        "swift sft "
        f"--model {args.model} "
        f"--dataset {args.dataset} "
        f"--output_dir {args.output} "
        "--train_type lora"
    )
    print(cmd)


if __name__ == "__main__":
    main()
