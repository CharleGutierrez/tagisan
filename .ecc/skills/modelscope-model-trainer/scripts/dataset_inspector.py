#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2"]
# ///

"""Inspect a ModelScope dataset snapshot and print split/file statistics."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from modelscope.hub.snapshot_download import dataset_snapshot_download


def scan_files(root: Path) -> dict[str, int]:
    counts = {"parquet": 0, "jsonl": 0, "json": 0, "csv": 0, "other": 0}
    for file in root.rglob("*"):
        if not file.is_file():
            continue
        suffix = file.suffix.lower()
        if suffix == ".parquet":
            counts["parquet"] += 1
        elif suffix == ".jsonl":
            counts["jsonl"] += 1
        elif suffix == ".json":
            counts["json"] += 1
        elif suffix == ".csv":
            counts["csv"] += 1
        else:
            counts["other"] += 1
    return counts


def main() -> None:
    parser = argparse.ArgumentParser(description="Inspect a ModelScope dataset snapshot")
    parser.add_argument("--dataset", required=True, help="Dataset repo id, e.g. org/name")
    parser.add_argument("--revision", default="master", help="Dataset revision")
    parser.add_argument("--local-dir", default="./data/inspector", help="Download directory")
    args = parser.parse_args()

    local_dir = dataset_snapshot_download(
        args.dataset,
        revision=args.revision,
        local_dir=args.local_dir,
    )
    root = Path(local_dir)

    payload = {
        "dataset": args.dataset,
        "revision": args.revision,
        "local_dir": str(root),
        "file_counts": scan_files(root),
    }
    print(json.dumps(payload, indent=2))


if __name__ == "__main__":
    main()
