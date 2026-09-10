#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2", "pyyaml>=6.0.0"]
# ///

"""Manage benchmark metadata for ModelScope model repositories."""

from __future__ import annotations

import argparse
import json
import re
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import yaml
from modelscope.hub.api import HubApi
from modelscope.hub.file_download import model_file_download

BENCHMARK_ALIASES = {
    "mmlu": ["mmlu", "massive multitask"],
    "gsm8k": ["gsm8k"],
    "arc": ["arc", "arc challenge"],
    "humaneval": ["humaneval", "human eval"],
    "mbpp": ["mbpp"],
}


def parse_markdown_tables(markdown: str) -> list[tuple[list[str], list[list[str]]]]:
    tables: list[tuple[list[str], list[list[str]]]] = []
    blocks = re.findall(r"(\|.+\|\n\|[-:| ]+\|\n(?:\|.*\|\n?)*)", markdown)
    for block in blocks:
        lines = [line.strip() for line in block.strip().splitlines() if line.strip()]
        if len(lines) < 2:
            continue
        headers = [c.strip() for c in lines[0].split("|")[1:-1]]
        rows = []
        for line in lines[2:]:
            rows.append([c.strip() for c in line.split("|")[1:-1]])
        if headers and rows:
            tables.append((headers, rows))
    return tables


def normalize_benchmark(name: str) -> str | None:
    lowered = name.lower()
    for key, aliases in BENCHMARK_ALIASES.items():
        if any(alias in lowered for alias in aliases):
            return key
    return None


def parse_score(text: str) -> float | None:
    text = text.strip().replace("%", "")
    try:
        return float(text)
    except ValueError:
        return None


def extract_from_markdown(markdown: str, repo_id: str) -> list[dict[str, Any]]:
    rows_out: list[dict[str, Any]] = []
    for headers, rows in parse_markdown_tables(markdown):
        if len(headers) < 2:
            continue
        bench_col = 0
        score_col = 1
        for idx, header in enumerate(headers):
            if "benchmark" in header.lower() or "task" in header.lower():
                bench_col = idx
            if "score" in header.lower() or "acc" in header.lower() or "value" in header.lower():
                score_col = idx
        for row in rows:
            if max(bench_col, score_col) >= len(row):
                continue
            bench_name = row[bench_col]
            bench_key = normalize_benchmark(bench_name) or bench_name.lower().replace(" ", "_")
            score = parse_score(row[score_col])
            if score is None:
                continue
            rows_out.append(
                {
                    "repo_id": repo_id,
                    "benchmark": bench_name,
                    "benchmark_key": bench_key,
                    "score": score,
                    "collected_at": datetime.now(timezone.utc).isoformat(),
                    "source": "readme_table",
                }
            )
    return rows_out


def download_readme(repo_id: str, token: str | None) -> str:
    with tempfile.TemporaryDirectory() as tmp:
        local = model_file_download(repo_id, "README.md", local_dir=tmp, token=token)
        return Path(local).read_text(encoding="utf-8", errors="ignore")


def cmd_extract_readme(args: argparse.Namespace) -> None:
    markdown = (
        Path(args.readme_file).read_text(encoding="utf-8")
        if args.readme_file
        else download_readme(args.repo_id, args.token)
    )
    rows = extract_from_markdown(markdown, args.repo_id)
    payload = {"repo_id": args.repo_id, "entries": rows}
    print(json.dumps(payload, indent=2))
    if args.output:
        Path(args.output).write_text(json.dumps(payload, indent=2), encoding="utf-8")


def cmd_import_json(args: argparse.Namespace) -> None:
    data = json.loads(Path(args.input).read_text(encoding="utf-8"))
    if isinstance(data, dict) and "entries" in data:
        entries = data["entries"]
    elif isinstance(data, list):
        entries = data
    else:
        raise ValueError("Unsupported JSON schema")

    required = {"benchmark", "score"}
    for idx, item in enumerate(entries):
        missing = required - set(item.keys())
        if missing:
            raise ValueError(f"Entry {idx} missing fields: {missing}")

    print(json.dumps({"valid_entries": len(entries)}, indent=2))

    if args.repo_id and not args.dry_run:
        api = HubApi()
        payload = json.dumps({"entries": entries}, indent=2).encode("utf-8")
        api.upload_file(
            path_or_fileobj=payload,
            path_in_repo="eval/leaderboard.json",
            repo_id=args.repo_id,
            repo_type="model",
            token=args.token,
            commit_message="Update evaluation leaderboard entries",
        )
        print(f"Uploaded evaluation file to {args.repo_id}:eval/leaderboard.json")


def cmd_to_yaml(args: argparse.Namespace) -> None:
    data = json.loads(Path(args.input).read_text(encoding="utf-8"))
    print(yaml.safe_dump(data, sort_keys=False, allow_unicode=False))


def main() -> None:
    parser = argparse.ArgumentParser(description="ModelScope evaluation metadata manager")
    sub = parser.add_subparsers(dest="command", required=True)

    p_extract = sub.add_parser("extract-readme")
    p_extract.add_argument("--repo-id", required=True)
    p_extract.add_argument("--readme-file", default=None)
    p_extract.add_argument("--token", default=None)
    p_extract.add_argument("--output", default=None)

    p_import = sub.add_parser("import-json")
    p_import.add_argument("--input", required=True)
    p_import.add_argument("--repo-id", default=None)
    p_import.add_argument("--token", default=None)
    p_import.add_argument("--dry-run", action="store_true")

    p_yaml = sub.add_parser("to-yaml")
    p_yaml.add_argument("--input", required=True)

    args = parser.parse_args()

    if args.command == "extract-readme":
        cmd_extract_readme(args)
    elif args.command == "import-json":
        cmd_import_json(args)
    else:
        cmd_to_yaml(args)


if __name__ == "__main__":
    main()
