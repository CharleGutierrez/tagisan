#!/usr/bin/env python3
"""Back up a source file into the project-root 备份 directory.

Rules:
- Create <project_root>/备份 if missing.
- Backup filename format: <source-name-without-suffixes>-YYYYMMDD-HHMMSS<all_suffixes>
- Keep at most 3 backups for the same source file.
- On the 4th backup, delete the oldest existing backup first.
"""

from __future__ import annotations

import argparse
import re
import shutil
from datetime import datetime, timedelta
from pathlib import Path


def split_name(path: Path) -> tuple[str, str]:
    suffix = "".join(path.suffixes)
    if suffix:
        return path.name[: -len(suffix)], suffix
    return path.name, ""


def build_target_path(backup_dir: Path, source: Path) -> Path:
    stem, suffix = split_name(source)
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    candidate = backup_dir / f"{stem}-{timestamp}{suffix}"

    if candidate.exists():
        for seconds in range(1, 60):
            shifted = datetime.now() + timedelta(seconds=seconds)
            timestamp = shifted.strftime("%Y%m%d-%H%M%S")
            candidate = backup_dir / f"{stem}-{timestamp}{suffix}"
            if not candidate.exists():
                break

    return candidate


def list_existing_backups(backup_dir: Path, source: Path) -> list[Path]:
    stem, suffix = split_name(source)
    pattern = re.compile(
        rf"^{re.escape(stem)}-\d{{8}}-\d{{6}}{re.escape(suffix)}$"
    )
    matches = [
        path
        for path in backup_dir.iterdir()
        if path.is_file() and pattern.match(path.name)
    ]
    return sorted(matches, key=lambda path: path.name)


def backup_source_file(project_root: Path, source_file: Path) -> Path:
    if not project_root.exists() or not project_root.is_dir():
        raise ValueError(f"Project root does not exist or is not a directory: {project_root}")

    if not source_file.exists() or not source_file.is_file():
        raise ValueError(f"Source file does not exist or is not a file: {source_file}")

    try:
        source_file.relative_to(project_root)
    except ValueError as exc:
        raise ValueError("Source file must be inside the project root") from exc

    backup_dir = project_root / "备份"
    backup_dir.mkdir(parents=True, exist_ok=True)

    existing_backups = list_existing_backups(backup_dir, source_file)
    while len(existing_backups) >= 3:
        oldest = existing_backups.pop(0)
        oldest.unlink()

    target = build_target_path(backup_dir, source_file)
    shutil.copy2(source_file, target)
    return target


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Back up a source file into the project-root 备份 directory."
    )
    parser.add_argument(
        "--project-root",
        required=True,
        help="Project root directory. The 备份 folder will be created here.",
    )
    parser.add_argument(
        "--source-file",
        required=True,
        help="Source file to back up. It must be inside the project root.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    project_root = Path(args.project_root).resolve()
    source_file = Path(args.source_file).resolve()

    target = backup_source_file(project_root, source_file)
    print(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
