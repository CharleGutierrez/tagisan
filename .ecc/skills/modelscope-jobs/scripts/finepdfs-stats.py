#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["duckdb>=1.0.0", "modelscope>=1.16.0", "packaging>=23.2"]
# ///

"""Compute dataset statistics and publish a summary artifact."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import duckdb
from modelscope.hub.api import HubApi
from modelscope.hub.snapshot_download import dataset_snapshot_download


def main() -> None:
    parser = argparse.ArgumentParser(description="Compute basic dataset stats")
    parser.add_argument("--source-dataset", required=True)
    parser.add_argument("--output-repo", required=True)
    parser.add_argument("--token", default=None)
    args = parser.parse_args()

    local = dataset_snapshot_download(args.source_dataset, local_dir="./data/stats-source")
    parquet_files = [str(p) for p in Path(local).rglob("*.parquet")]
    jsonl_files = [str(p) for p in Path(local).rglob("*.jsonl")]

    summary: dict[str, object] = {
        "source_dataset": args.source_dataset,
        "parquet_files": len(parquet_files),
        "jsonl_files": len(jsonl_files),
    }

    if parquet_files:
        conn = duckdb.connect()
        rows = conn.execute(f"SELECT count(*) FROM read_parquet({json.dumps(parquet_files)})").fetchone()[0]
        summary["parquet_rows"] = int(rows)

    out = Path("./outputs/finepdfs-stats.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(json.dumps(summary, indent=2))

    api = HubApi()
    api.create_repo(args.output_repo, repo_type="dataset", token=args.token, exist_ok=True)
    api.upload_file(
        path_or_fileobj=str(out),
        path_in_repo="reports/finepdfs_stats.json",
        repo_id=args.output_repo,
        repo_type="dataset",
        token=args.token,
        commit_message="Upload dataset statistics report",
    )
    print(f"Uploaded report to {args.output_repo}")


if __name__ == "__main__":
    main()
