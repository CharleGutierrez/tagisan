#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = ["modelscope>=1.16.0", "packaging>=23.2"]
# ///

"""Convert local model artifacts to GGUF and optionally upload to ModelScope."""

from __future__ import annotations

import argparse
import shutil
import subprocess
from pathlib import Path

from modelscope.hub.api import HubApi


def run_convert(converter: str, model_dir: Path, output_file: Path) -> None:
    cmd = [converter, str(model_dir), str(output_file)]
    subprocess.run(cmd, check=True)


def main() -> None:
    parser = argparse.ArgumentParser(description="Convert a model directory to GGUF")
    parser.add_argument("--converter", required=True, help="Path to conversion executable/script")
    parser.add_argument("--model-dir", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--upload-repo", default=None, help="Optional ModelScope model repo id")
    parser.add_argument("--token", default=None)
    args = parser.parse_args()

    model_dir = Path(args.model_dir).resolve()
    output_file = Path(args.output).resolve()
    output_file.parent.mkdir(parents=True, exist_ok=True)

    if not shutil.which(args.converter) and not Path(args.converter).exists():
        raise FileNotFoundError(f"Converter not found: {args.converter}")

    run_convert(args.converter, model_dir, output_file)
    print(f"GGUF written to {output_file}")

    if args.upload_repo:
        api = HubApi()
        api.upload_file(
            path_or_fileobj=str(output_file),
            path_in_repo=output_file.name,
            repo_id=args.upload_repo,
            repo_type="model",
            token=args.token,
            commit_message="Upload GGUF artifact",
        )
        print(f"Uploaded to {args.upload_repo}")


if __name__ == "__main__":
    main()
