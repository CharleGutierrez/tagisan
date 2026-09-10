#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Generate a structured SAM-style segmentation training plan."""

from __future__ import annotations

import argparse
import json


def main() -> None:
    parser = argparse.ArgumentParser(description="Print a SAM segmentation training plan")
    parser.add_argument("--model", required=True)
    parser.add_argument("--train-dir", required=True)
    parser.add_argument("--val-dir", required=True)
    parser.add_argument("--output", default="./outputs/sam")
    parser.add_argument("--epochs", default="10")
    parser.add_argument("--batch-size", default="4")
    parser.add_argument("--lr", default="1e-4")
    args = parser.parse_args()

    payload = {
        "task": "segmentation",
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
