#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2"]
# ///

"""Create and publish paper-style markdown documents for ModelScope repos."""

from __future__ import annotations

import argparse
from datetime import date
from pathlib import Path

from modelscope.hub.api import HubApi

ROOT = Path(__file__).resolve().parent.parent
TEMPLATES = ROOT / "templates"


def render_template(template_name: str, fields: dict[str, str]) -> str:
    path = TEMPLATES / f"{template_name}.md"
    if not path.exists():
        raise FileNotFoundError(f"Template not found: {template_name}")
    text = path.read_text(encoding="utf-8")
    for key, value in fields.items():
        text = text.replace(f"{{{{{key}}}}}", value)
    return text


def build_fields(args: argparse.Namespace) -> dict[str, str]:
    fallback_url = "https://www.modelscope.cn"
    project_url = args.url or fallback_url
    return {
        "TITLE": args.title,
        "AUTHORS": args.authors,
        "ABSTRACT": args.abstract,
        "URL": project_url,
        "DATE": args.date or date.today().isoformat(),
        "ARXIV_URL": args.arxiv_url or project_url,
        "PDF_URL": args.pdf_url or project_url,
        "CODE_URL": args.code_url or "https://github.com/hyf020908/modelscope-skills",
        "DEMO_URL": args.demo_url or "https://www.modelscope.cn/studios",
        "MODEL_URL": args.model_url or "https://www.modelscope.cn/models",
        "DATASET_URL": args.dataset_url or "https://www.modelscope.cn/datasets",
    }


def create_paper(args: argparse.Namespace) -> None:
    fields = build_fields(args)
    content = render_template(args.template, fields)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(content, encoding="utf-8")
    print(f"Created: {output}")


def upload_paper(args: argparse.Namespace) -> None:
    api = HubApi()
    api.upload_file(
        path_or_fileobj=args.input,
        path_in_repo=args.path_in_repo,
        repo_id=args.repo_id,
        repo_type="model",
        token=args.token,
        commit_message=f"Add paper report: {Path(args.input).name}",
    )
    print(f"Uploaded {args.input} to {args.repo_id}:{args.path_in_repo}")


def list_templates() -> None:
    for path in sorted(TEMPLATES.glob("*.md")):
        print(path.stem)


def main() -> None:
    parser = argparse.ArgumentParser(description="Paper markdown manager for ModelScope repos")
    sub = parser.add_subparsers(dest="command", required=True)

    p_create = sub.add_parser("create", help="Create a paper-style markdown file from a template.")
    p_create.add_argument("--title", required=True)
    p_create.add_argument("--authors", required=True)
    p_create.add_argument("--abstract", default="")
    p_create.add_argument("--url", default="")
    p_create.add_argument("--date", default="")
    p_create.add_argument("--arxiv-url", default="")
    p_create.add_argument("--pdf-url", default="")
    p_create.add_argument("--code-url", default="")
    p_create.add_argument("--demo-url", default="")
    p_create.add_argument("--model-url", default="")
    p_create.add_argument("--dataset-url", default="")
    p_create.add_argument("--template", default="standard")
    p_create.add_argument("--output", required=True)

    p_upload = sub.add_parser("upload", help="Upload a rendered markdown paper to a ModelScope model repo.")
    p_upload.add_argument("--repo-id", required=True)
    p_upload.add_argument("--input", required=True)
    p_upload.add_argument("--path-in-repo", required=True)
    p_upload.add_argument("--token", default=None)

    sub.add_parser("list-templates", help="List available paper templates.")

    args = parser.parse_args()

    if args.command == "create":
        create_paper(args)
    elif args.command == "upload":
        upload_paper(args)
    else:
        list_templates()


if __name__ == "__main__":
    main()
