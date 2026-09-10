#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2"]
# ///

"""Create and maintain dataset repositories on ModelScope Hub."""

from __future__ import annotations

import argparse
import json
import tempfile
import time
from pathlib import Path
from typing import Any

from modelscope.hub.api import HubApi
from modelscope.hub.file_download import dataset_file_download

ROOT = Path(__file__).resolve().parent.parent
TEMPLATES_DIR = ROOT / "templates"


def load_template(name: str) -> dict[str, Any]:
    path = TEMPLATES_DIR / f"{name}.json"
    if not path.exists():
        raise FileNotFoundError(f"Template not found: {name}")
    return json.loads(path.read_text(encoding="utf-8"))


def validate_rows(rows: list[dict[str, Any]], template_name: str) -> None:
    template = load_template(template_name)
    schema = template.get("validation_schema", {})
    required = schema.get("required_fields", [])
    for idx, row in enumerate(rows):
        missing = [field for field in required if field not in row]
        if missing:
            raise ValueError(f"Row {idx} missing required fields: {missing}")


def init_repo(api: HubApi, repo_id: str, token: str | None, visibility: str) -> None:
    api.create_repo(repo_id, repo_type="dataset", token=token, exist_ok=True, visibility=visibility)

    readme = f"""---
license: apache-2.0
---

# {repo_id.split('/')[-1]}

Dataset managed by `modelscope-datasets/scripts/dataset_manager.py`.
"""
    api.upload_file(
        path_or_fileobj=readme.encode("utf-8"),
        path_in_repo="README.md",
        repo_id=repo_id,
        repo_type="dataset",
        token=token,
        commit_message="Initialize dataset README",
    )


def add_rows(
    api: HubApi,
    repo_id: str,
    rows: list[dict[str, Any]],
    split: str,
    token: str | None,
    append: bool,
) -> None:
    path_in_repo = f"data/{split}.jsonl"
    merged_rows = list(rows)

    if append:
        with tempfile.TemporaryDirectory() as tmp:
            try:
                existing = dataset_file_download(repo_id, path_in_repo, local_dir=tmp, token=token)
                for line in Path(existing).read_text(encoding="utf-8").splitlines():
                    if line.strip():
                        merged_rows.insert(0, json.loads(line))
            except Exception:
                pass

    payload = "\n".join(json.dumps(item, ensure_ascii=False) for item in merged_rows) + "\n"

    api.upload_file(
        path_or_fileobj=payload.encode("utf-8"),
        path_in_repo=path_in_repo,
        repo_id=repo_id,
        repo_type="dataset",
        token=token,
        commit_message=f"Update split '{split}' with {len(rows)} rows",
    )


def print_stats(api: HubApi, repo_id: str, split: str, token: str | None) -> None:
    path_in_repo = f"data/{split}.jsonl"
    with tempfile.TemporaryDirectory() as tmp:
        local = dataset_file_download(repo_id, path_in_repo, local_dir=tmp, token=token)
        lines = [line for line in Path(local).read_text(encoding="utf-8").splitlines() if line.strip()]
        print(json.dumps({"repo_id": repo_id, "split": split, "rows": len(lines)}, indent=2))


def main() -> None:
    parser = argparse.ArgumentParser(description="ModelScope dataset manager")
    sub = parser.add_subparsers(dest="command", required=True)

    p_init = sub.add_parser("init", help="Initialize dataset repository")
    p_init.add_argument("--repo-id", required=True)
    p_init.add_argument("--token", default=None)
    p_init.add_argument("--visibility", choices=["public", "private", "internal"], default="public")

    p_quick = sub.add_parser("quick-setup", help="Initialize repository and upload template metadata")
    p_quick.add_argument("--repo-id", required=True)
    p_quick.add_argument("--template", required=True)
    p_quick.add_argument("--token", default=None)
    p_quick.add_argument("--visibility", choices=["public", "private", "internal"], default="public")

    p_add = sub.add_parser("add-rows", help="Upload JSON rows to a split")
    p_add.add_argument("--repo-id", required=True)
    p_add.add_argument("--template", default="chat")
    p_add.add_argument("--split", default="train")
    p_add.add_argument("--rows-json", default=None)
    p_add.add_argument("--rows-file", default=None)
    p_add.add_argument("--append", action="store_true")
    p_add.add_argument("--token", default=None)

    p_stats = sub.add_parser("stats", help="Read split row statistics")
    p_stats.add_argument("--repo-id", required=True)
    p_stats.add_argument("--split", default="train")
    p_stats.add_argument("--token", default=None)

    sub.add_parser("list-templates", help="List available templates")

    args = parser.parse_args()
    api = HubApi()

    if args.command == "list-templates":
        for path in sorted(TEMPLATES_DIR.glob("*.json")):
            payload = json.loads(path.read_text(encoding="utf-8"))
            print(f"{path.stem}: {payload.get('description', 'no description')}")
        return

    if args.command == "init":
        init_repo(api, args.repo_id, args.token, args.visibility)
        print(f"Initialized dataset repo: {args.repo_id}")
        return

    if args.command == "quick-setup":
        init_repo(api, args.repo_id, args.token, args.visibility)
        template = load_template(args.template)
        config = {
            "template": args.template,
            "created_at": time.time(),
            "template_description": template.get("description", ""),
        }
        api.upload_file(
            path_or_fileobj=json.dumps(config, indent=2).encode("utf-8"),
            path_in_repo="config.json",
            repo_id=args.repo_id,
            repo_type="dataset",
            token=args.token,
            commit_message="Add dataset template configuration",
        )
        print(f"Quick setup complete for {args.repo_id} with template {args.template}")
        return

    if args.command == "add-rows":
        if not args.rows_json and not args.rows_file:
            raise ValueError("Provide --rows-json or --rows-file")
        if args.rows_json:
            rows = json.loads(args.rows_json)
        else:
            rows = json.loads(Path(args.rows_file).read_text(encoding="utf-8"))
        if not isinstance(rows, list):
            raise ValueError("Rows must be a JSON array")
        validate_rows(rows, args.template)
        add_rows(api, args.repo_id, rows, args.split, args.token, args.append)
        print(f"Uploaded {len(rows)} rows to {args.repo_id}:{args.split}")
        return

    if args.command == "stats":
        print_stats(api, args.repo_id, args.split, args.token)
        return


if __name__ == "__main__":
    main()
