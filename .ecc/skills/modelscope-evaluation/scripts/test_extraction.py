#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2", "pyyaml>=6.0.0"]
# ///

"""Basic extraction tests for evaluation_manager table parsing."""

from __future__ import annotations

import sys
import unittest

sys.dont_write_bytecode = True

from evaluation_manager import extract_from_markdown


class ExtractionTests(unittest.TestCase):
    def test_extract_simple_table(self) -> None:
        md = """
| Benchmark | Score |
|---|---|
| MMLU | 73.2 |
| GSM8K | 89.1 |
"""
        rows = extract_from_markdown(md, "org/model")
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0]["benchmark_key"], "mmlu")


if __name__ == "__main__":
    unittest.main()
