#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2"]
# ///

"""Generate simple response records from prompt datasets and optionally publish."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from modelscope.hub.api import HubApi
from modelscope.hub.snapshot_download import dataset_snapshot_download


def load_prompts(path: Path, max_rows: int) -> list[str]:
    rows: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        obj = json.loads(line)
        prompt = obj.get("prompt") or obj.get("text") or obj.get("instruction")
        if prompt:
            rows.append(str(prompt))
        if len(rows) >= max_rows:
            break
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate responses for prompt datasets")
    parser.add_argument("--source-dataset", required=True)
    parser.add_argument("--output-dataset", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--limit", type=int, default=200)
    parser.add_argument("--token", default=None)
    args = parser.parse_args()

    local_dir = dataset_snapshot_download(args.source_dataset, local_dir="./data/jobs-source")
    candidates = sorted(Path(local_dir).rglob("*.jsonl"))
    if not candidates:
        raise FileNotFoundError("No JSONL prompt files found in source dataset")

    prompts = load_prompts(candidates[0], args.limit)
    outputs = [
        {
            "prompt": prompt,
            "response": f"[template-response via {args.model}] {prompt[:160]}",
            "model": args.model,
        }
        for prompt in prompts
    ]

    out_path = Path("./outputs/generate-responses.jsonl")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(json.dumps(item, ensure_ascii=False) for item in outputs) + "\n", encoding="utf-8")
    print(f"Wrote {len(outputs)} rows to {out_path}")

    api = HubApi()
    api.create_repo(args.output_dataset, repo_type="dataset", token=args.token, exist_ok=True)
    api.upload_file(
        path_or_fileobj=str(out_path),
        path_in_repo="data/generated.jsonl",
        repo_id=args.output_dataset,
        repo_type="dataset",
        token=args.token,
        commit_message="Upload generated responses",
    )
    print(f"Uploaded results to {args.output_dataset}")


if __name__ == "__main__":
    main()
