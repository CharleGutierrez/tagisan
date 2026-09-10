#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["requests>=2.32.0"]
# ///

"""Fetch benchmark rows from an external API and print normalized JSON."""

from __future__ import annotations

import argparse
import json
import os

import requests


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Fetch benchmark rows from Artificial Analysis and print normalized JSON."
    )
    parser.add_argument("--api-key", default=None, help="Artificial Analysis API key. Defaults to AA_API_KEY env var.")
    parser.add_argument("--limit", type=int, default=5, help="Maximum number of rows to print.")
    args = parser.parse_args()

    api_key = args.api_key or os.getenv("AA_API_KEY")
    if not api_key:
        parser.error("API key is required. Provide --api-key or set AA_API_KEY.")

    url = "https://artificialanalysis.ai/api/v2/data/llms/models"
    resp = requests.get(url, headers={"x-api-key": api_key}, timeout=30)
    resp.raise_for_status()
    data = resp.json()

    rows = []
    for item in data[: args.limit]:
        rows.append({
            "benchmark": item.get("name", "unknown"),
            "score": item.get("intelligence_index", 0),
            "source": "artificial_analysis",
        })

    print(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
