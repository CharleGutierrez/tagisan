#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2"]
# ///

"""Create synthetic chain-of-thought style records from prompt inputs."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from modelscope.hub.api import HubApi


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate synthetic CoT instruction dataset")
    parser.add_argument("--input", required=True, help="Local JSONL with prompt field")
    parser.add_argument("--output-dataset", required=True)
    parser.add_argument("--token", default=None)
    parser.add_argument("--limit", type=int, default=200)
    args = parser.parse_args()

    src = Path(args.input)
    rows = []
    for line in src.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        obj = json.loads(line)
        prompt = obj.get("prompt") or obj.get("instruction") or obj.get("text")
        if not prompt:
            continue
        rows.append(
            {
                "prompt": prompt,
                "analysis": "Break the task into smaller reasoning steps before giving final answer.",
                "answer": f"Template answer for: {prompt[:120]}",
            }
        )
        if len(rows) >= args.limit:
            break

    out = Path("./outputs/cot-self-instruct.jsonl")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(json.dumps(x, ensure_ascii=False) for x in rows) + "\n", encoding="utf-8")
    print(f"Generated {len(rows)} rows")

    api = HubApi()
    api.create_repo(args.output_dataset, repo_type="dataset", token=args.token, exist_ok=True)
    api.upload_file(
        path_or_fileobj=str(out),
        path_in_repo="data/cot_self_instruct.jsonl",
        repo_id=args.output_dataset,
        repo_type="dataset",
        token=args.token,
        commit_message="Upload CoT self-instruct dataset",
    )
    print(f"Uploaded to {args.output_dataset}")


if __name__ == "__main__":
    main()
