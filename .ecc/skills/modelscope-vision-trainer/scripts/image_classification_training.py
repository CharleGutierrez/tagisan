#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Generate a structured image classification training plan."""

from __future__ import annotations

import argparse
import json


def main() -> None:
    parser = argparse.ArgumentParser(description="Print a classification training plan")
    parser.add_argument("--model", required=True)
    parser.add_argument("--train-dir", required=True)
    parser.add_argument("--val-dir", required=True)
    parser.add_argument("--output", default="./outputs/cls")
    parser.add_argument("--epochs", default="5")
    parser.add_argument("--batch-size", default="16")
    parser.add_argument("--lr", default="3e-4")
    args = parser.parse_args()

    payload = {
        "task": "classification",
        "model_id": args.model,
        "train_dir": args.train_dir,
        "val_dir": args.val_dir,
        "output_dir": args.output,
        "epochs": args.epochs,
        "batch_size": args.batch_size,
        "learning_rate": args.lr,
        "status": "plan_ready",
    }
    print(json.dumps(payload, indent=2))


if __name__ == "__main__":
    main()
