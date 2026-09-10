#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Generate a robust `swift sft` command template for local or remote workflows."""

from __future__ import annotations

import argparse
import shlex


def main() -> None:
    parser = argparse.ArgumentParser(description="Print a safer ms-swift SFT command template")
    parser.add_argument("--model", required=True, help="Model ID or local checkpoint")
    parser.add_argument("--dataset", required=True, help="Training dataset path or hub dataset")
    parser.add_argument("--output", default="./outputs/sft", help="Output directory")
    parser.add_argument("--val-dataset", default="", help="Optional validation dataset")
    parser.add_argument("--train-type", default="lora", choices=["lora", "qlora", "full"])
    parser.add_argument("--max-length", type=int, default=512)
    parser.add_argument("--batch-size", type=int, default=1)
    parser.add_argument("--grad-acc", type=int, default=4)
    parser.add_argument("--lr", type=float, default=5e-5)
    parser.add_argument("--epochs", type=int, default=1)
    parser.add_argument("--max-steps", type=int, default=24)
    parser.add_argument("--save-steps", type=int, default=12)
    parser.add_argument("--eval-steps", type=int, default=12)
    parser.add_argument("--logging-steps", type=int, default=5)
    parser.add_argument("--lora-rank", type=int, default=8)
    parser.add_argument("--system", default="", help="Optional default system prompt")
    parser.add_argument(
        "--device-mode",
        choices=["cpu-safe", "gpu-auto"],
        default="cpu-safe",
        help="cpu-safe disables bf16/fp16 and forces float32",
    )
    args = parser.parse_args()

    cmd_parts = [
        "swift sft",
        f"--model {shlex.quote(args.model)}",
        f"--dataset {shlex.quote(args.dataset)}",
        f"--train_type {shlex.quote(args.train_type)}",
        f"--output_dir {shlex.quote(args.output)}",
        f"--max_length {args.max_length}",
        f"--per_device_train_batch_size {args.batch_size}",
        f"--gradient_accumulation_steps {args.grad_acc}",
        f"--learning_rate {args.lr}",
        f"--num_train_epochs {args.epochs}",
        f"--max_steps {args.max_steps}",
        f"--save_steps {args.save_steps}",
        f"--eval_steps {args.eval_steps}",
        f"--logging_steps {args.logging_steps}",
    ]
    if args.train_type in {"lora", "qlora"}:
        cmd_parts.append(f"--lora_rank {args.lora_rank}")
    if args.val_dataset:
        cmd_parts.append(f"--val_dataset {shlex.quote(args.val_dataset)}")
    if args.system:
        cmd_parts.append(f"--system {shlex.quote(args.system)}")
    if args.device_mode == "cpu-safe":
        cmd_parts.extend(["--bf16 false", "--fp16 false", "--torch_dtype float32"])
    print(" ".join(cmd_parts))


if __name__ == "__main__":
    main()
