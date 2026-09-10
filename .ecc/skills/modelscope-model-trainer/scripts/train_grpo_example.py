#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Generate a recommended `swift rlhf --rlhf_type grpo` command."""

from __future__ import annotations

import argparse
import shlex


def main() -> None:
    parser = argparse.ArgumentParser(description="Print an ms-swift GRPO command template")
    parser.add_argument("--model", required=True)
    parser.add_argument("--dataset", required=True)
    parser.add_argument("--output", default="./outputs/grpo")
    parser.add_argument("--train-type", default="lora", choices=["lora", "qlora", "full"])
    parser.add_argument("--use-vllm", default="false")
    args = parser.parse_args()

    cmd = (
        "swift rlhf "
        "--rlhf_type grpo "
        f"--model {shlex.quote(args.model)} "
        f"--dataset {shlex.quote(args.dataset)} "
        f"--output_dir {shlex.quote(args.output)} "
        f"--train_type {shlex.quote(args.train_type)} "
        f"--use_vllm {shlex.quote(args.use_vllm)}"
    )
    print(cmd)


if __name__ == "__main__":
    main()
