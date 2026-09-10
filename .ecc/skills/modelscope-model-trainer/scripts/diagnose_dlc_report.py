#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Parse a DLC diagnosis report and output concrete auto-fix patches."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Dict, List


def read_text(path: str) -> str:
    p = Path(path)
    if not p.exists():
        raise FileNotFoundError(path)
    return p.read_text(encoding="utf-8", errors="ignore")


def detect(text: str) -> Dict[str, bool]:
    t = text.lower()
    return {
        "resource_limit": "resourcelimit" in t or "超过配额" in text or "gpu资源数量超过配额" in text,
        "bf16_unsupported": "doesn't support bf16" in t or "不支持 bf16" in text or "bf16/gpu" in t,
        "enter_running_timeout": "enterrunningtimeout" in t or "envpreparing" in t or "imagepullbackoff" in t,
        "install_only_exit": (
            "成功安装了ms-swift" in text
            and ("没有执行任何训练" in text or "缺少实际训练代码执行" in text)
        )
        or ("pip成功安装" in text and "没有执行任何python脚本" in text),
    }


def patch_suggestions(flags: Dict[str, bool]) -> Dict[str, str]:
    env_patch: Dict[str, str] = {}
    pilot_patch: Dict[str, str] = {}
    notes: List[str] = []

    if flags["resource_limit"]:
        env_patch.update({"PAI_DLC_GPU": "0", "PAI_DLC_CPU": "4", "PAI_DLC_MEMORY": "16Gi"})
        notes.append("Switch to CPU route to bypass GPU quota threshold.")
    if flags["bf16_unsupported"]:
        pilot_patch["EXTRA_SWIFT_ARGS"] = "--bf16 false --fp16 false --torch_dtype float32"
        notes.append("Disable bf16/fp16 and force float32.")
    if flags["enter_running_timeout"]:
        env_patch["PAI_DLC_IMAGE"] = "pytorch/pytorch:latest"
        notes.append("Use a verified public image tag to avoid pull timeout.")
    if flags["install_only_exit"]:
        notes.append("Keep startup command short and ensure it explicitly executes swift training.")

    return {
        "env_patch": env_patch,
        "pilot_patch": pilot_patch,
        "full_patch": pilot_patch.copy(),
        "notes": " ".join(notes) if notes else "No known high-confidence pattern matched.",
    }


def replace_kv(path: Path, patch: Dict[str, str]) -> bool:
    if not path.exists() or not patch:
        return False
    lines = path.read_text(encoding="utf-8").splitlines()
    found = {k: False for k in patch}
    out: List[str] = []
    for raw in lines:
        line = raw
        if "=" in line and not line.lstrip().startswith("#"):
            k, _ = line.split("=", 1)
            key = k.strip()
            if key in patch:
                line = f"{key}={patch[key]}"
                found[key] = True
        out.append(line)
    for key, seen in found.items():
        if not seen:
            out.append(f"{key}={patch[key]}")
    path.write_text("\n".join(out) + "\n", encoding="utf-8")
    return True


def main() -> None:
    parser = argparse.ArgumentParser(description="Diagnose DLC report and output fix patch")
    parser.add_argument("--report-file", required=True, help="Path to markdown/text diagnosis report")
    parser.add_argument("--apply", action="store_true", help="Apply patch to configs/*.env when present")
    parser.add_argument("--root", default=".", help="Workspace root for apply mode")
    args = parser.parse_args()

    text = read_text(args.report_file)
    job_id_match = re.search(r"任务ID[^A-Za-z0-9]*([A-Za-z0-9]+)", text) or re.search(
        r"Task\\s*ID[^A-Za-z0-9]*([A-Za-z0-9]+)", text, re.IGNORECASE
    )
    job_id = job_id_match.group(1) if job_id_match else ""
    flags = detect(text)
    patch = patch_suggestions(flags)

    applied: List[str] = []
    if args.apply:
        root = Path(args.root).resolve()
        if replace_kv(root / "configs/remote.auto.env", patch["env_patch"]):
            applied.append(str(root / "configs/remote.auto.env"))
        if replace_kv(root / "configs/pilot.env", patch["pilot_patch"]):
            applied.append(str(root / "configs/pilot.env"))
        if replace_kv(root / "configs/full.env", patch["full_patch"]):
            applied.append(str(root / "configs/full.env"))

    result = {
        "job_id": job_id,
        "matched_flags": flags,
        "patch": patch,
        "applied_files": applied,
        "next_commands": [
            ".venv/bin/python scripts/submit_pai_dlc.py --env-file configs/remote.auto.env --mode pilot --wait --retry-once",
            ".venv/bin/python scripts/submit_pai_dlc.py --env-file configs/remote.auto.env --mode full --wait",
        ],
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
