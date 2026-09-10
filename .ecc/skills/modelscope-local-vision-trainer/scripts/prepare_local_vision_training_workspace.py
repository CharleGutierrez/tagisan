#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Prepare local vision training configs and a launch script behind a continue gate."""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shlex
from pathlib import Path
from typing import Any


TASK_DEFAULTS = {
    "classification": {
        "model_id": "google/vit-base-patch16-224",
        "train_dir": "./data/vision/train",
        "val_dir": "./data/vision/val",
        "epochs": "5",
        "batch_size": "16",
        "learning_rate": "3e-4",
        "image_size": "224",
        "entrypoint": "train_image_classification.py",
    },
    "detection": {
        "model_id": "facebook/detr-resnet-50",
        "train_annotations": "./data/annotations/train.json",
        "val_annotations": "./data/annotations/val.json",
        "images_root": "./data/images",
        "epochs": "12",
        "batch_size": "4",
        "learning_rate": "1e-4",
        "image_size": "800",
        "entrypoint": "train_object_detection.py",
    },
    "segmentation": {
        "model_id": "facebook/sam-vit-base",
        "train_dir": "./data/vision/train",
        "val_dir": "./data/vision/val",
        "epochs": "10",
        "batch_size": "4",
        "learning_rate": "1e-4",
        "image_size": "1024",
        "entrypoint": "train_sam_segmentation.py",
    },
}


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def parse_env_file(path: Path) -> dict[str, str]:
    data: dict[str, str] = {}
    if not path.exists():
        return data
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        raw_value = value.strip()
        try:
            parts = shlex.split(raw_value, posix=True)
            data[key.strip()] = parts[0] if len(parts) == 1 else raw_value
        except ValueError:
            data[key.strip()] = raw_value
    return data


def write_env_file(path: Path, mapping: dict[str, str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = [f"{key}={shlex.quote(str(mapping.get(key, '')))}" for key in mapping if key.upper() == key]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_shell_script(path: Path, root: Path) -> None:
    required_env = (root / "configs" / "local.vision.required.env").relative_to(root).as_posix()
    train_env = (root / "configs" / "local.vision.train.env").relative_to(root).as_posix()
    content = "\n".join(
        [
            "#!/usr/bin/env bash",
            "set -euo pipefail",
            "",
            f"source {required_env}",
            f"source {train_env}",
            "",
            'if [[ "${LOCAL_HOST_OS}" != "linux" ]]; then',
            '  echo "LOCAL_HOST_OS must be linux before local vision training starts." >&2',
            "  exit 1",
            "fi",
            'if ! command -v "${PYTHON_BIN}" >/dev/null 2>&1; then',
            '  echo "Configured PYTHON_BIN is not available on this machine." >&2',
            "  exit 1",
            "fi",
            'if [[ ! -f "${LOCAL_VISION_ENTRYPOINT}" ]]; then',
            '  echo "Configured LOCAL_VISION_ENTRYPOINT does not exist." >&2',
            "  exit 1",
            "fi",
            "",
            'cd "${LOCAL_WORKDIR}"',
            'export CUDA_VISIBLE_DEVICES="${CUDA_VISIBLE_DEVICES}"',
            'export NPROC_PER_NODE="${NPROC_PER_NODE}"',
            "",
            'CMD=("${PYTHON_BIN}" "${LOCAL_VISION_ENTRYPOINT}" "--model" "${MODEL_ID}" "--output" "${OUTPUT_DIR}")',
            'if [[ "${VISION_TASK}" == "detection" ]]; then',
            '  CMD+=("--train-annotations" "${TRAIN_ANNOTATIONS}" "--val-annotations" "${VAL_ANNOTATIONS}" "--images-root" "${IMAGES_ROOT}")',
            "else",
            '  CMD+=("--train-dir" "${TRAIN_DIR}" "--val-dir" "${VAL_DIR}")',
            "fi",
            'if [[ -n "${EPOCHS:-}" ]]; then',
            '  CMD+=("--epochs" "${EPOCHS}")',
            "fi",
            'if [[ -n "${BATCH_SIZE:-}" ]]; then',
            '  CMD+=("--batch-size" "${BATCH_SIZE}")',
            "fi",
            'if [[ -n "${LEARNING_RATE:-}" ]]; then',
            '  CMD+=("--lr" "${LEARNING_RATE}")',
            "fi",
            'if [[ -n "${IMAGE_SIZE:-}" ]]; then',
            '  CMD+=("--image-size" "${IMAGE_SIZE}")',
            "fi",
            "",
            '"${CMD[@]}"',
            "",
        ]
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    path.chmod(path.stat().st_mode | 0o111)


def detect_runtime() -> dict[str, Any]:
    system = platform.system().lower()
    return {
        "system": system,
        "is_linux": system == "linux",
    }


def detect_task(request: str) -> str:
    lowered = request.lower()
    if any(token in lowered for token in ["detection", "detect", "目标检测"]):
        return "detection"
    if any(token in lowered for token in ["segment", "segmentation", "分割", "sam"]):
        return "segmentation"
    return "classification"


def is_probable_repo_id(token: str) -> bool:
    lowered = token.lower()
    if token.startswith((".", "/", "~")):
        return False
    bad_suffixes = (
        ".json",
        ".jsonl",
        ".csv",
        ".tsv",
        ".parquet",
        ".txt",
        ".jpg",
        ".jpeg",
        ".png",
        ".webp",
    )
    return not lowered.endswith(bad_suffixes)


def extract_repo_like(request: str, keywords: list[str]) -> str:
    lowered = request.lower()
    repo_pat = re.compile(r"(?<![/.\w-])([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)")
    for keyword in keywords:
        idx = lowered.find(keyword.lower())
        if idx == -1:
            continue
        snippet = request[idx : idx + 160]
        match = repo_pat.search(snippet)
        if match and is_probable_repo_id(match.group(1)):
            return match.group(1)
    return ""


def extract_value(request: str, keys: list[str]) -> str:
    for key in keys:
        match = re.search(rf"{re.escape(key)}\s*[:=：]\s*([^\s,，;；]+)", request, re.IGNORECASE)
        if match:
            return match.group(1)
    return ""


def build_required_env(root: Path, runtime: dict[str, Any], entrypoint: str) -> dict[str, str]:
    existing = parse_env_file(root / "configs" / "local.vision.required.env")
    return {
        "LOCAL_HOST_OS": existing.get("LOCAL_HOST_OS", "linux" if runtime["is_linux"] else runtime["system"] or "linux"),
        "LOCAL_WORKDIR": existing.get("LOCAL_WORKDIR", str(root)),
        "PYTHON_BIN": existing.get("PYTHON_BIN", "python"),
        "CUDA_VISIBLE_DEVICES": existing.get("CUDA_VISIBLE_DEVICES", os.environ.get("CUDA_VISIBLE_DEVICES", "0")),
        "NPROC_PER_NODE": existing.get("NPROC_PER_NODE", os.environ.get("NPROC_PER_NODE", "1")),
        "LOCAL_VISION_ENTRYPOINT": existing.get("LOCAL_VISION_ENTRYPOINT", entrypoint),
        "TRAIN_LOG_DIR": existing.get("TRAIN_LOG_DIR", str(root / "outputs" / "logs")),
        "TRAIN_HOST_NOTE": existing.get("TRAIN_HOST_NOTE", "review_then_continue"),
    }


def build_train_env(task: str, request: str) -> dict[str, str]:
    defaults = TASK_DEFAULTS[task]
    env = {
        "VISION_TASK": task,
        "MODEL_ID": extract_repo_like(request, ["模型", "model"]) or defaults["model_id"],
        "OUTPUT_DIR": f"./outputs/local-{task}",
        "EPOCHS": extract_value(request, ["epochs", "epoch", "轮数"]) or defaults["epochs"],
        "BATCH_SIZE": extract_value(request, ["batch size", "batch_size", "批大小"]) or defaults["batch_size"],
        "LEARNING_RATE": extract_value(request, ["learning rate", "learning_rate", "lr", "学习率"]) or defaults["learning_rate"],
        "IMAGE_SIZE": extract_value(request, ["image_size", "image size", "图像尺寸"]) or defaults["image_size"],
        "LOCAL_VISION_NOT_EXECUTED": "true",
    }
    if task == "detection":
        env["TRAIN_ANNOTATIONS"] = extract_value(request, ["train_annotations", "训练标注"]) or defaults["train_annotations"]
        env["VAL_ANNOTATIONS"] = extract_value(request, ["val_annotations", "验证标注"]) or defaults["val_annotations"]
        env["IMAGES_ROOT"] = extract_value(request, ["images_root", "图像目录"]) or defaults["images_root"]
    else:
        env["TRAIN_DIR"] = extract_value(request, ["train_dir", "训练目录"]) or defaults["train_dir"]
        env["VAL_DIR"] = extract_value(request, ["val_dir", "验证目录"]) or defaults["val_dir"]
    return env


def build_plan(task: str, train_env: dict[str, str], required_env_path: Path, train_env_path: Path, launch_path: Path, runtime: dict[str, Any]) -> dict[str, Any]:
    return {
        "task": task,
        "generated_files": {
            "required_env_file": str(required_env_path),
            "train_env_file": str(train_env_path),
            "launch_script": str(launch_path),
        },
        "runtime_check": runtime,
        "wait_for_continue": True,
        "continue_gate_file": str(required_env_path),
        "launch_command": f"bash {launch_path.name}",
        "prepared_only": True,
        "not_executed": True,
        "train_summary": train_env,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Prepare local vision training configs and a launch script")
    parser.add_argument("--request", default="", help="Natural-language vision training request")
    parser.add_argument("--root", default=".", help="Workspace root")
    parser.add_argument("--task", choices=["classification", "detection", "segmentation"], help="Override task")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    request = (args.request or "").strip()
    task = args.task or detect_task(request)
    defaults = TASK_DEFAULTS[task]
    runtime = detect_runtime()

    configs_dir = root / "configs"
    scripts_dir = root / "scripts"
    required_env_path = configs_dir / "local.vision.required.env"
    train_env_path = configs_dir / "local.vision.train.env"
    launch_path = scripts_dir / "run_local_vision_training.sh"

    required_env = build_required_env(root, runtime, defaults["entrypoint"])
    train_env = build_train_env(task, request)

    write_env_file(required_env_path, required_env)
    write_env_file(train_env_path, train_env)
    write_shell_script(launch_path, root)

    plan = build_plan(task, train_env, required_env_path, train_env_path, launch_path, runtime)
    write_json(configs_dir / "local.vision.training.json", plan)

    result = {
        "workspace_root": str(root),
        "task": task,
        "generated_files": [
            str(configs_dir / "local.vision.training.json"),
            str(required_env_path),
            str(train_env_path),
            str(launch_path),
        ],
        "wait_for_continue": True,
        "continue_gate_file": str(required_env_path),
        "launch_command": f"bash {launch_path.relative_to(root).as_posix()}",
        "runtime_check": runtime,
        "prepared_only": True,
        "not_executed": True,
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
