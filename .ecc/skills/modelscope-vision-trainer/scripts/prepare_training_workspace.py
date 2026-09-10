#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Prepare a natural-language-first vision training workspace."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


PAI_KEYS = [
    "ALIBABA_CLOUD_ACCESS_KEY_ID",
    "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
    "PAI_REGION",
    "PAI_WORKSPACE_ID",
]

TASK_DEFAULTS = {
    "classification": {
        "model_id": "google/vit-base-patch16-224",
        "train_dir": "./data/vision/train",
        "val_dir": "./data/vision/val",
        "epochs": "5",
        "batch_size": "16",
        "learning_rate": "3e-4",
        "image_size": "224",
        "command": "uv run scripts/image_classification_training.py",
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
        "command": "uv run scripts/object_detection_training.py",
    },
    "segmentation": {
        "model_id": "facebook/sam-vit-base",
        "train_dir": "./data/vision/train",
        "val_dir": "./data/vision/val",
        "epochs": "10",
        "batch_size": "4",
        "learning_rate": "1e-4",
        "image_size": "1024",
        "command": "uv run scripts/sam_segmentation_training.py",
    },
}


def parse_env_file(path: Path) -> dict[str, str]:
    data: dict[str, str] = {}
    if not path.exists():
        return data
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        data[key.strip()] = value.strip()
    return data


def csv_items(raw: str) -> list[str]:
    return [part.strip() for part in raw.split(",") if part.strip()]


def join_csv_items(items: list[str]) -> str:
    ordered: list[str] = []
    seen: set[str] = set()
    for item in items:
        if not item or item in seen:
            continue
        ordered.append(item)
        seen.add(item)
    return ",".join(ordered)


def write_env_file(path: Path, mapping: dict[str, str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(f"{key}={mapping.get(key, '')}" for key in mapping) + "\n", encoding="utf-8")


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def resolve_local_path(raw: str, root: Path) -> Path:
    candidate = Path(raw).expanduser()
    if candidate.is_absolute():
        return candidate.resolve()
    return (root / candidate).resolve()


def path_in_repo(root: Path, item: Path) -> str:
    try:
        rel = item.resolve().relative_to(root.resolve())
        return rel.as_posix()
    except ValueError:
        return item.name


def looks_like_local_path(raw: str, root: Path) -> bool:
    if not raw:
        return False
    if raw.startswith(("./", "../", "/", "~/")):
        return True
    return resolve_local_path(raw, root).exists()


def keep_remote_asset_entry(entry: str, root: Path) -> bool:
    if entry in {"data", "configs", "scripts"}:
        return True
    if looks_like_local_path(entry, root):
        return resolve_local_path(entry, root).exists()
    return True


def extra_remote_assets(root: Path, values: list[str]) -> list[str]:
    assets: list[str] = []
    for value in values:
        if looks_like_local_path(value, root):
            resolved = resolve_local_path(value, root)
            assets.append(value if not Path(value).expanduser().is_absolute() else str(resolved))
    return assets


def detect_task(request: str) -> str:
    lowered = request.lower()
    if any(token in lowered for token in ["detection", "detect", "目标检测"]):
        return "detection"
    if any(token in lowered for token in ["segment", "segmentation", "分割", "sam"]):
        return "segmentation"
    return "classification"


def is_probable_repo_id(token: str) -> bool:
    lowered = token.lower()
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


def extract_model_id(request: str, default: str) -> str:
    explicit = extract_repo_like(request, ["模型", "基础模型", "base model"])
    if explicit:
        return explicit
    english = re.search(
        r"\bmodel\b\s*[:=：]?\s*([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)",
        request,
        re.IGNORECASE,
    )
    if english and is_probable_repo_id(english.group(1)):
        return english.group(1)
    return default


def extract_value(request: str, keys: list[str]) -> str:
    for key in keys:
        match = re.search(rf"{re.escape(key)}\s*[:=：]\s*([^\s,，;；]+)", request, re.IGNORECASE)
        if match:
            return match.group(1)
    return ""


def build_remote_env(existing: dict[str, str], repo_base: str, extra_assets: list[str], root: Path) -> dict[str, str]:
    remote_asset_paths = [
        entry
        for entry in csv_items(existing.get("REMOTE_ASSET_PATHS", "data,configs,scripts"))
        if keep_remote_asset_entry(entry, root)
    ]
    remote_asset_paths.extend(extra_assets)
    return {
        "ALIBABA_CLOUD_ACCESS_KEY_ID": existing.get("ALIBABA_CLOUD_ACCESS_KEY_ID", ""),
        "ALIBABA_CLOUD_ACCESS_KEY_SECRET": existing.get("ALIBABA_CLOUD_ACCESS_KEY_SECRET", ""),
        "ALIBABA_CLOUD_SECURITY_TOKEN": existing.get("ALIBABA_CLOUD_SECURITY_TOKEN", ""),
        "PAI_REGION": existing.get("PAI_REGION", "cn-shanghai"),
        "PAI_WORKSPACE_ID": existing.get("PAI_WORKSPACE_ID", ""),
        "PAI_DLC_IMAGE": existing.get("PAI_DLC_IMAGE", "pytorch/pytorch:2.4.0-cuda12.1-cudnn9-runtime"),
        "PAI_DLC_ECS_SPEC": existing.get("PAI_DLC_ECS_SPEC", ""),
        "PAI_DLC_JOB_TYPE": existing.get("PAI_DLC_JOB_TYPE", "PyTorchJob"),
        "PAI_DLC_ROLE_TYPE": existing.get("PAI_DLC_ROLE_TYPE", "Worker"),
        "PAI_DLC_POD_COUNT": existing.get("PAI_DLC_POD_COUNT", "1"),
        "PAI_DLC_GPU": existing.get("PAI_DLC_GPU", "1"),
        "PAI_DLC_CPU": existing.get("PAI_DLC_CPU", "8"),
        "PAI_DLC_MEMORY": existing.get("PAI_DLC_MEMORY", "32Gi"),
        "PAI_DLC_MAX_MINUTES": existing.get("PAI_DLC_MAX_MINUTES", "240"),
        "MODELSCOPE_API_TOKEN": existing.get("MODELSCOPE_API_TOKEN", ""),
        "MS_REPO_OWNER": existing.get("MS_REPO_OWNER", ""),
        "MS_REPO_BASE": existing.get("MS_REPO_BASE", repo_base),
        "REMOTE_ASSET_REPO": existing.get("REMOTE_ASSET_REPO", ""),
        "REMOTE_ASSET_PATHS": join_csv_items(remote_asset_paths),
        "REMOTE_BOOTSTRAP_COMMAND": existing.get(
            "REMOTE_BOOTSTRAP_COMMAND",
            'python -m pip install -U pip setuptools wheel && python -m pip install -U "modelscope>=1.23.0"',
        ),
    }


def maybe_write_pai_gate(root: Path, remote_env: dict[str, str]) -> bool:
    if all(remote_env.get(key, "").strip() for key in PAI_KEYS):
        return False
    write_env_file(
        root / "configs" / "pai.required.env",
        {
            "ALIBABA_CLOUD_ACCESS_KEY_ID": "",
            "ALIBABA_CLOUD_ACCESS_KEY_SECRET": "",
            "PAI_REGION": remote_env.get("PAI_REGION", "cn-shanghai"),
            "PAI_WORKSPACE_ID": "",
        },
    )
    return True


def main() -> None:
    parser = argparse.ArgumentParser(description="Prepare natural-language vision training configs")
    parser.add_argument("--request", default="", help="Natural-language training request")
    parser.add_argument("--root", default=".", help="Workspace root")
    parser.add_argument("--task", choices=["classification", "detection", "segmentation"], help="Override task")
    parser.add_argument("--force", action="store_true", help="Overwrite generated placeholder files when applicable")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    request = (args.request or "").strip()
    task = args.task or detect_task(request)
    defaults = TASK_DEFAULTS[task].copy()

    model_id = extract_model_id(request, defaults["model_id"])
    epochs = extract_value(request, ["epochs", "epoch", "轮数"]) or defaults["epochs"]
    batch_size = extract_value(request, ["batch size", "batch_size", "批大小"]) or defaults["batch_size"]
    learning_rate = extract_value(request, ["learning rate", "learning_rate", "lr", "学习率"]) or defaults["learning_rate"]
    image_size = extract_value(request, ["image_size", "image size", "图像尺寸"]) or defaults["image_size"]

    plan = {
        "request_text": request,
        "task": task,
        "model_id": model_id,
        "output_dir": f"./outputs/{task}",
        "epochs": epochs,
        "batch_size": batch_size,
        "learning_rate": learning_rate,
        "image_size": image_size,
    }

    if task == "detection":
        plan["train_annotations"] = extract_value(request, ["train_annotations", "训练标注"]) or defaults["train_annotations"]
        plan["val_annotations"] = extract_value(request, ["val_annotations", "验证标注"]) or defaults["val_annotations"]
        plan["images_root"] = extract_value(request, ["images_root", "图像目录"]) or defaults["images_root"]
        asset_candidates = [
            plan["train_annotations"],
            plan["val_annotations"],
            plan["images_root"],
        ]
    else:
        plan["train_dir"] = extract_value(request, ["train_dir", "训练目录"]) or defaults["train_dir"]
        plan["val_dir"] = extract_value(request, ["val_dir", "验证目录"]) or defaults["val_dir"]
        asset_candidates = [plan["train_dir"], plan["val_dir"]]

    configs_dir = root / "configs"
    configs_dir.mkdir(parents=True, exist_ok=True)
    write_json(configs_dir / "vision.training.json", plan)

    repo_base = re.sub(r"[^a-z0-9]+", "-", model_id.lower()).strip("-") or f"vision-{task}"
    existing_remote = parse_env_file(configs_dir / "remote.auto.env")
    remote_env = build_remote_env(
        existing_remote,
        f"{repo_base}-{task}",
        extra_remote_assets(root, asset_candidates),
        root,
    )
    write_env_file(configs_dir / "remote.auto.env", remote_env)
    gate_created = maybe_write_pai_gate(root, remote_env)

    result = {
        "workspace_root": str(root),
        "task": task,
        "model_id": model_id,
        "remote_asset_paths": csv_items(remote_env["REMOTE_ASSET_PATHS"]),
        "generated_files": [
            str(configs_dir / "vision.training.json"),
            str(configs_dir / "remote.auto.env"),
        ]
        + ([str(configs_dir / "pai.required.env")] if gate_created else []),
        "recommended_command": defaults["command"],
        "credential_gate_created": gate_created,
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
