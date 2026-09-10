#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Backward-compatible wrapper around the natural-language training planner."""

from __future__ import annotations

import argparse

from training_workspace import main as planner_main


def main() -> None:
    parser = argparse.ArgumentParser(description="Prepare training configs from a plain-language request")
    parser.add_argument("--root", default=".", help="Workspace root")
    parser.add_argument("--request", default="", help="Natural-language request")
    parser.add_argument("--force", action="store_true", help="Overwrite generated dataset files")
    args, _ = parser.parse_known_args()

    planner_main()


if __name__ == "__main__":
    main()
