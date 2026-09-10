#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Generate a structured object detection training plan."""

from __future__ import annotations

import argparse
import json


def main() -> None:
    parser = argparse.ArgumentParser(description="Print an object detection training plan")
    parser.add_argument("--model", required=True)
    parser.add_argument("--train-annotations", required=True)
    parser.add_argument("--val-annotations", required=True)
    parser.add_argument("--images-root", required=True)
    parser.add_argument("--output", default="./outputs/det")
    parser.add_argument("--epochs", default="12")
    parser.add_argument("--batch-size", default="4")
    parser.add_argument("--lr", default="1e-4")
    args = parser.parse_args()

    payload = {
        "task": "detection",
        "model_id": args.model,
        "train_annotations": args.train_annotations,
        "val_annotations": args.val_annotations,
        "images_root": args.images_root,
        "output_dir": args.output,
        "epochs": args.epochs,
        "batch_size": args.batch_size,
        "learning_rate": args.lr,
        "status": "plan_ready",
    }
    print(json.dumps(payload, indent=2))


if __name__ == "__main__":
    main()
