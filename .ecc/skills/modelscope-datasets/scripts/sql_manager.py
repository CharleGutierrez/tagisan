#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["duckdb>=1.0.0", "modelscope>=1.16.0", "packaging>=23.2", "pandas>=2.0.0", "pyarrow>=15.0.0"]
# ///

"""Run DuckDB SQL over local files or downloaded ModelScope dataset snapshots."""

from __future__ import annotations

import argparse
import json
import tempfile
from pathlib import Path
from typing import Iterable

import duckdb

from modelscope.hub.api import HubApi
from modelscope.hub.snapshot_download import dataset_snapshot_download


def discover_data_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for ext in ("*.parquet", "*.jsonl", "*.json", "*.csv"):
        files.extend(root.rglob(ext))
    return sorted({file.resolve() for file in files if file.is_file()})


def register_view(conn: duckdb.DuckDBPyConnection, files: list[Path], view_name: str = "data") -> None:
    if not files:
        raise ValueError("No input files found")

    parquet = [str(f) for f in files if f.suffix.lower() == ".parquet"]
    json_files = [str(f) for f in files if f.suffix.lower() in {".json", ".jsonl"}]
    csv_files = [str(f) for f in files if f.suffix.lower() == ".csv"]

    statements = []
    if parquet:
        statements.append(f"SELECT * FROM read_parquet({json.dumps(parquet)})")
    if json_files:
        statements.append(f"SELECT * FROM read_json_auto({json.dumps(json_files)})")
    if csv_files:
        statements.append(f"SELECT * FROM read_csv_auto({json.dumps(csv_files)})")

    union_sql = " UNION ALL ".join(statements)
    conn.execute(f"CREATE OR REPLACE VIEW {view_name} AS {union_sql}")


def resolve_input(dataset: str | None, input_glob: str | None) -> tuple[Path, list[Path]]:
    if input_glob:
        matches = [Path(p) for p in sorted(Path().glob(input_glob))]
        return Path("."), [m.resolve() for m in matches if m.is_file()]

    if dataset:
        with tempfile.TemporaryDirectory() as tmp:
            local_dir = dataset_snapshot_download(dataset, local_dir=tmp)
            root = Path(local_dir)
            files = discover_data_files(root)
            cache_root = Path("./data/sql-manager-cache") / dataset.replace("/", "__")
            cache_root.mkdir(parents=True, exist_ok=True)
            for file in files:
                target = cache_root / file.name
                target.write_bytes(file.read_bytes())
            return cache_root, discover_data_files(cache_root)

    raise ValueError("Provide --dataset or --input")


def save_query_result(conn: duckdb.DuckDBPyConnection, sql: str, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    if output.suffix.lower() == ".parquet":
        conn.execute(f"COPY ({sql}) TO '{output}' (FORMAT PARQUET)")
    else:
        conn.execute(f"COPY ({sql}) TO '{output}' (HEADER, DELIMITER ',')")


def push_file(repo_id: str, local_file: Path, token: str | None) -> None:
    api = HubApi()
    api.create_repo(repo_id, repo_type="dataset", token=token, exist_ok=True)
    api.upload_file(
        path_or_fileobj=str(local_file),
        path_in_repo=f"data/{local_file.name}",
        repo_id=repo_id,
        repo_type="dataset",
        token=token,
        commit_message=f"Upload SQL output {local_file.name}",
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="DuckDB SQL manager for ModelScope datasets")
    sub = parser.add_subparsers(dest="command", required=True)

    p_query = sub.add_parser("query")
    p_query.add_argument("--dataset", default=None)
    p_query.add_argument("--input", default=None, help="Glob path, e.g. './data/**/*.parquet'")
    p_query.add_argument("--sql", required=True)
    p_query.add_argument("--limit", type=int, default=20)

    p_describe = sub.add_parser("describe")
    p_describe.add_argument("--dataset", default=None)
    p_describe.add_argument("--input", default=None)

    p_export = sub.add_parser("export")
    p_export.add_argument("--dataset", default=None)
    p_export.add_argument("--input", default=None)
    p_export.add_argument("--sql", required=True)
    p_export.add_argument("--output", required=True)
    p_export.add_argument("--push-to", default=None)
    p_export.add_argument("--token", default=None)

    args = parser.parse_args()

    root, files = resolve_input(getattr(args, "dataset", None), getattr(args, "input", None))
    conn = duckdb.connect()
    register_view(conn, files)

    if args.command == "describe":
        rows = conn.execute("DESCRIBE SELECT * FROM data").fetchall()
        print(json.dumps([{"column": r[0], "type": r[1]} for r in rows], indent=2))
        return

    if args.command == "query":
        sql = args.sql
        if "limit" not in sql.lower():
            sql = f"{sql} LIMIT {args.limit}"
        rows = conn.execute(sql).fetchdf()
        print(rows.to_string(index=False))
        return

    if args.command == "export":
        output = Path(args.output).resolve()
        save_query_result(conn, args.sql, output)
        print(f"Saved: {output}")
        if args.push_to:
            push_file(args.push_to, output, args.token)
            print(f"Uploaded to dataset repo: {args.push_to}")
        return

    raise RuntimeError("Unknown command")


if __name__ == "__main__":
    main()
