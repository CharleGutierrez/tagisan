#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Inspect a local vision dataset directory."""

from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path

IMAGE_EXT = {".jpg", ".jpeg", ".png", ".bmp", ".webp"}


def main() -> None:
    parser = argparse.ArgumentParser(description="Inspect class-folder image datasets")
    parser.add_argument("--root", required=True, help="Root directory with class subfolders")
    args = parser.parse_args()

    root = Path(args.root)
    if not root.exists():
        raise FileNotFoundError(root)

    class_counts: Counter[str] = Counter()
    for class_dir in root.iterdir():
        if not class_dir.is_dir():
            continue
        count = sum(1 for f in class_dir.rglob("*") if f.suffix.lower() in IMAGE_EXT)
        class_counts[class_dir.name] = count

    payload = {
        "root": str(root),
        "num_classes": len(class_counts),
        "total_images": sum(class_counts.values()),
        "class_counts": dict(class_counts),
    }
    print(json.dumps(payload, indent=2))


if __name__ == "__main__":
    main()
