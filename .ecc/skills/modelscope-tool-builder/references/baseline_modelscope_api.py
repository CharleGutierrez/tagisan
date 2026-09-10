#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["requests>=2.32.0"]
# ///

"""Minimal ModelScope model search script."""

from __future__ import annotations

import argparse
import os

import requests


API = "https://www.modelscope.cn/openapi/v1/models"


def main() -> None:
    parser = argparse.ArgumentParser(description="List models from ModelScope OpenAPI")
    parser.add_argument("--author", default=None)
    parser.add_argument("--search", default=None)
    parser.add_argument("--page-size", type=int, default=20)
    parser.add_argument("--page-number", type=int, default=1)
    args = parser.parse_args()

    headers = {}
    token = os.getenv("MODELSCOPE_API_TOKEN")
    if token:
        headers["Authorization"] = f"Bearer {token}"

    params = {
        "page_size": args.page_size,
        "page_number": args.page_number,
    }
    if args.author:
        params["author"] = args.author
    if args.search:
        params["search"] = args.search

    resp = requests.get(API, params=params, headers=headers, timeout=30)
    resp.raise_for_status()
    print(resp.text)


if __name__ == "__main__":
    main()
