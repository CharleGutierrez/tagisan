#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Turn a plain-language training request into reproducible config files."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


DEFAULT_MODEL = "Qwen/Qwen2.5-0.5B-Instruct"
DEFAULT_SYSTEM = "You are a helpful assistant."
PAI_KEYS = [
    "ALIBABA_CLOUD_ACCESS_KEY_ID",
    "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
    "PAI_REGION",
    "PAI_WORKSPACE_ID",
]

METHOD_DEFAULTS: dict[str, dict[str, str]] = {
    "sft": {
        "train_type": "lora",
        "max_length": "2048",
        "batch_size": "1",
        "grad_acc": "4",
        "learning_rate": "5e-5",
        "epochs": "1",
        "max_steps": "24",
        "save_steps": "12",
        "eval_steps": "12",
        "logging_steps": "5",
        "lora_rank": "8",
        "extra_swift_args": "--torch_dtype bfloat16",
    },
    "dpo": {
        "train_type": "lora",
        "max_length": "2048",
        "batch_size": "1",
        "grad_acc": "4",
        "learning_rate": "1e-5",
        "epochs": "1",
        "max_steps": "24",
        "save_steps": "12",
        "eval_steps": "12",
        "logging_steps": "5",
        "lora_rank": "8",
        "beta": "0.1",
        "loss_type": "sigmoid",
        "extra_swift_args": "--torch_dtype bfloat16",
    },
    "grpo": {
        "train_type": "lora",
        "max_length": "2048",
        "batch_size": "1",
        "grad_acc": "8",
        "learning_rate": "1e-6",
        "epochs": "1",
        "max_steps": "24",
        "save_steps": "12",
        "eval_steps": "12",
        "logging_steps": "5",
        "lora_rank": "8",
        "use_vllm": "false",
        "extra_swift_args": "--torch_dtype bfloat16",
    },
}

PILOT_OVERRIDES: dict[str, dict[str, str]] = {
    "sft": {
        "MAX_STEPS": "6",
        "SAVE_STEPS": "3",
        "EVAL_STEPS": "3",
        "LOGGING_STEPS": "2",
        "MAX_LENGTH": "1024",
        "LORA_RANK": "4",
    },
    "dpo": {
        "MAX_STEPS": "6",
        "SAVE_STEPS": "3",
        "EVAL_STEPS": "3",
        "LOGGING_STEPS": "2",
        "MAX_LENGTH": "1024",
        "LORA_RANK": "4",
    },
    "grpo": {
        "MAX_STEPS": "6",
        "SAVE_STEPS": "3",
        "EVAL_STEPS": "3",
        "LOGGING_STEPS": "2",
        "MAX_LENGTH": "1024",
        "LORA_RANK": "4",
        "USE_VLLM": "false",
    },
}

KEY_ALIASES = {
    "learning rate": "learning_rate",
    "learning_rate": "learning_rate",
    "lr": "learning_rate",
    "学习率": "learning_rate",
    "batch size": "batch_size",
    "batch_size": "batch_size",
    "per_device_train_batch_size": "batch_size",
    "训练批大小": "batch_size",
    "批大小": "batch_size",
    "grad acc": "grad_acc",
    "grad_acc": "grad_acc",
    "gradient accumulation": "grad_acc",
    "gradient_accumulation_steps": "grad_acc",
    "梯度累积": "grad_acc",
    "epochs": "epochs",
    "epoch": "epochs",
    "轮数": "epochs",
    "max steps": "max_steps",
    "max_steps": "max_steps",
    "步数": "max_steps",
    "save steps": "save_steps",
    "save_steps": "save_steps",
    "eval steps": "eval_steps",
    "eval_steps": "eval_steps",
    "logging steps": "logging_steps",
    "logging_steps": "logging_steps",
    "max length": "max_length",
    "max_length": "max_length",
    "上下文长度": "max_length",
    "序列长度": "max_length",
    "lora rank": "lora_rank",
    "lora_rank": "lora_rank",
    "beta": "beta",
    "loss type": "loss_type",
    "loss_type": "loss_type",
    "train type": "train_type",
    "train_type": "train_type",
    "微调方式": "train_type",
    "use vllm": "use_vllm",
    "use_vllm": "use_vllm",
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
    lines = [f"{key}={mapping.get(key, '')}" for key in mapping if key.upper() == key]
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_jsonl_if_missing(path: Path, rows: list[dict[str, Any]], force: bool) -> bool:
    if path.exists() and not force:
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    content = "\n".join(json.dumps(row, ensure_ascii=False) for row in rows) + "\n"
    path.write_text(content, encoding="utf-8")
    return True


def sanitize_repo_base(model_id: str, method: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", model_id.lower()).strip("-")
    return f"{slug or 'modelscope-run'}-{method}"


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


def map_dataset_for_remote(raw: str, root: Path) -> tuple[str, list[str], str]:
    if not raw or not looks_like_local_path(raw, root):
        return raw, [], ""
    resolved = resolve_local_path(raw, root)
    entry = raw if not Path(raw).expanduser().is_absolute() else str(resolved)
    remote_path = f"./remote_assets/{path_in_repo(root, resolved)}"
    return remote_path, [entry], str(resolved)


def normalize_method(raw: str) -> str:
    text = raw.lower()
    if any(token in text for token in ["grpo", "rlhf", "强化学习", "reinforcement"]):
        return "grpo"
    if any(token in text for token in ["dpo", "偏好优化", "preference optimization"]):
        return "dpo"
    return "sft"


def detect_train_type(request: str) -> str:
    lowered = request.lower()
    if "qlora" in lowered:
        return "qlora"
    if "全参数" in request or "full parameter" in lowered or re.search(r"\bfull\b", lowered):
        return "full"
    return "lora"


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
    repo_pat = re.compile(r"(?<![/.\w-])([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(?:#[A-Za-z0-9_.-]+)?)")
    for keyword in keywords:
        idx = lowered.find(keyword.lower())
        if idx == -1:
            continue
        snippet = request[idx : idx + 160]
        match = repo_pat.search(snippet)
        if match and is_probable_repo_id(match.group(1)):
            return match.group(1)
    return ""


def extract_first_repo_like(request: str) -> str:
    repo_pat = re.compile(r"(?<![/.\w-])([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+(?:#[A-Za-z0-9_.-]+)?)")
    for match in repo_pat.finditer(request):
        token = match.group(1)
        if is_probable_repo_id(token):
            return token
    return ""


def extract_path_like(request: str, keywords: list[str]) -> str:
    lowered = request.lower()
    path_pat = re.compile(
        r"(?:^|[\s'\"=：:])(?P<path>\./[^\s,，;；]+|\.\./[^\s,，;；]+|/[^\s,，;；]+|~/[^\s,，;；]+)"
    )
    for keyword in keywords:
        idx = lowered.find(keyword.lower())
        if idx == -1:
            continue
        snippet = request[idx : idx + 160]
        match = path_pat.search(snippet)
        if match:
            return match.group("path")
    return ""


def extract_first_local_path(request: str) -> str:
    path_pat = re.compile(
        r"(?:^|[\s'\"=：:])(?P<path>\./[^\s,，;；]+|\.\./[^\s,，;；]+|/[^\s,，;；]+|~/[^\s,，;；]+)"
    )
    match = path_pat.search(request)
    return match.group("path") if match else ""


def extract_scalar_params(request: str) -> dict[str, str]:
    params: dict[str, str] = {}
    pattern = re.compile(
        r"(?P<key>learning rate|learning_rate|lr|学习率|batch size|batch_size|per_device_train_batch_size|"
        r"训练批大小|批大小|grad acc|grad_acc|gradient accumulation|gradient_accumulation_steps|梯度累积|"
        r"epochs|epoch|轮数|max steps|max_steps|步数|save steps|save_steps|eval steps|eval_steps|"
        r"logging steps|logging_steps|max length|max_length|上下文长度|序列长度|lora rank|lora_rank|beta|"
        r"loss type|loss_type|train type|train_type|微调方式|use vllm|use_vllm)"
        r"\s*[:=：]\s*(?P<value>[^\s,，;；]+)",
        re.IGNORECASE,
    )
    for match in pattern.finditer(request):
        key = KEY_ALIASES[match.group("key").lower()]
        params[key] = match.group("value")
    return params


def build_messages(user_text: str, assistant_text: str) -> list[dict[str, str]]:
    return [
        {"role": "system", "content": DEFAULT_SYSTEM},
        {"role": "user", "content": user_text},
        {"role": "assistant", "content": assistant_text},
    ]


def default_rows(method: str) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    if method == "dpo":
        train = [
            {
                "messages": build_messages(
                    "Rewrite this message in a more professional tone: send me the file now.",
                    "Could you please send me the file when you have a moment?",
                ),
                "rejected_response": "Send me the file now.",
            },
            {
                "messages": build_messages(
                    "Answer briefly: why are unit tests useful?",
                    "Unit tests catch regressions early and make refactoring safer.",
                ),
                "rejected_response": "They are useful.",
            },
        ]
        valid = [
            {
                "messages": build_messages(
                    "Summarize this in one line: keep code small and well tested.",
                    "Small, well-tested code is easier to maintain and debug.",
                ),
                "rejected_response": "Write better code.",
            }
        ]
        return train, valid

    if method == "grpo":
        train = [
            {
                "messages": [
                    {"role": "system", "content": "You are a careful math assistant."},
                    {"role": "user", "content": "What is 12 + 17? Show the final answer only."},
                ],
                "solution": "29",
            },
            {
                "messages": [
                    {"role": "system", "content": "You are a careful math assistant."},
                    {"role": "user", "content": "Compute 9 * 8. Final answer only."},
                ],
                "solution": "72",
            },
        ]
        valid = [
            {
                "messages": [
                    {"role": "system", "content": "You are a careful math assistant."},
                    {"role": "user", "content": "What is 15 - 6? Final answer only."},
                ],
                "solution": "9",
            }
        ]
        return train, valid

    train = [
        {
            "messages": build_messages(
                "Explain supervised fine-tuning in one sentence.",
                "Supervised fine-tuning adapts a base model on example inputs and target outputs.",
            )
        },
        {
            "messages": build_messages(
                "Translate to Chinese: We will review the experiment results tomorrow.",
                "我们明天会复盘实验结果。",
            )
        },
        {
            "messages": build_messages(
                "Rewrite with a polite tone: send the report today.",
                "Could you please send the report today?",
            )
        },
    ]
    valid = [
        {
            "messages": build_messages(
                "Give one reason for using LoRA.",
                "LoRA reduces training cost by updating only a small set of adapter weights.",
            )
        }
    ]
    return train, valid


def ensure_default_dataset(root: Path, method: str, force: bool) -> dict[str, Any]:
    train_rows, valid_rows = default_rows(method)
    data_dir = root / "data" / method
    train_path = data_dir / "train.jsonl"
    valid_path = data_dir / "valid.jsonl"
    changed = []
    if write_jsonl_if_missing(train_path, train_rows, force):
        changed.append(str(train_path))
    if write_jsonl_if_missing(valid_path, valid_rows, force):
        changed.append(str(valid_path))
    return {
        "train_path": train_path,
        "valid_path": valid_path,
        "changed_files": changed,
    }


def build_remote_env(
    existing: dict[str, str], repo_base: str, extra_asset_paths: list[str], root: Path
) -> dict[str, str]:
    remote_asset_paths = [
        entry
        for entry in csv_items(existing.get("REMOTE_ASSET_PATHS", "data,configs"))
        if keep_remote_asset_entry(entry, root)
    ]
    remote_asset_paths.extend(extra_asset_paths)
    remote = {
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
            'python -m pip install -U pip setuptools wheel && python -m pip install -U "modelscope>=1.23.0" "ms-swift>=3.8"',
        ),
    }
    return remote


def maybe_write_pai_gate(root: Path, remote_env: dict[str, str]) -> bool:
    if all(remote_env.get(key, "").strip() for key in PAI_KEYS):
        return False
    target = root / "configs" / "pai.required.env"
    write_env_file(
        target,
        {
            "ALIBABA_CLOUD_ACCESS_KEY_ID": "",
            "ALIBABA_CLOUD_ACCESS_KEY_SECRET": "",
            "PAI_REGION": remote_env.get("PAI_REGION", "cn-shanghai"),
            "PAI_WORKSPACE_ID": "",
        },
    )
    return True


def build_training_request(args: argparse.Namespace) -> dict[str, Any]:
    request = (args.request or "").strip()
    method = args.method or normalize_method(request)
    model_id = args.model or extract_repo_like(request, ["模型", "model", "base model", "基础模型"])
    if not model_id:
        model_id = extract_first_repo_like(request)
    if not model_id:
        model_id = DEFAULT_MODEL

    dataset = args.dataset or extract_repo_like(request, ["数据集", "dataset"])
    if not dataset:
        dataset = extract_path_like(request, ["数据集", "dataset", "训练集"])
    if not dataset:
        dataset = extract_first_local_path(request)

    params = METHOD_DEFAULTS[method].copy()
    params.update(extract_scalar_params(request))
    if "train_type" not in params or not params["train_type"]:
        params["train_type"] = detect_train_type(request)
    elif params["train_type"].lower() not in {"lora", "qlora", "full"}:
        params["train_type"] = detect_train_type(request)

    output_dir = args.output_dir or f"./outputs/{method}"
    return {
        "request_text": request,
        "method": method,
        "model_id": model_id,
        "dataset": dataset,
        "output_dir": output_dir,
        "system_prompt": DEFAULT_SYSTEM,
        "params": params,
    }


def training_env_from_request(
    request_cfg: dict[str, Any], train_dataset: str, valid_dataset: str | None
) -> dict[str, str]:
    params = request_cfg["params"]
    env = {
        "MODEL_ID": request_cfg["model_id"],
        "TRAINER_METHOD": request_cfg["method"],
        "TRAIN_DATASET": train_dataset,
        "OUTPUT_DIR": request_cfg["output_dir"],
        "TRAIN_TYPE": params["train_type"],
        "MAX_LENGTH": params["max_length"],
        "BATCH_SIZE": params["batch_size"],
        "GRAD_ACC": params["grad_acc"],
        "LEARNING_RATE": params["learning_rate"],
        "EPOCHS": params["epochs"],
        "MAX_STEPS": params["max_steps"],
        "SAVE_STEPS": params["save_steps"],
        "EVAL_STEPS": params["eval_steps"],
        "LOGGING_STEPS": params["logging_steps"],
        "LORA_RANK": params["lora_rank"],
        "SYSTEM_PROMPT": request_cfg["system_prompt"],
        "EXTRA_SWIFT_ARGS": params["extra_swift_args"],
    }
    if valid_dataset:
        env["VALID_DATASET"] = valid_dataset
    if request_cfg["method"] == "dpo":
        env["BETA"] = params["beta"]
        env["LOSS_TYPE"] = params["loss_type"]
    if request_cfg["method"] == "grpo":
        env["USE_VLLM"] = params["use_vllm"]
    return env


def main() -> None:
    parser = argparse.ArgumentParser(description="Prepare natural-language training configs")
    parser.add_argument("--request", default="", help="Natural-language training request")
    parser.add_argument("--root", default=".", help="Workspace root")
    parser.add_argument("--method", choices=["sft", "dpo", "grpo"], help="Override method")
    parser.add_argument("--model", default="", help="Override model id")
    parser.add_argument("--dataset", default="", help="Override dataset id or local path")
    parser.add_argument("--output-dir", default="", help="Override output directory")
    parser.add_argument("--force", action="store_true", help="Overwrite generated dataset files")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    request_cfg = build_training_request(args)
    dataset_bootstrap = None
    dataset = request_cfg["dataset"]
    valid_dataset = ""
    changed_files: list[str] = []
    extra_asset_paths: list[str] = []
    local_dataset_source = ""

    if dataset:
        train_dataset, asset_paths, local_dataset_source = map_dataset_for_remote(dataset, root)
        extra_asset_paths.extend(asset_paths)
    else:
        dataset_bootstrap = ensure_default_dataset(root, request_cfg["method"], force=args.force)
        changed_files.extend(dataset_bootstrap["changed_files"])
        train_dataset = f"./remote_assets/{dataset_bootstrap['train_path'].relative_to(root).as_posix()}"
        valid_dataset = f"./remote_assets/{dataset_bootstrap['valid_path'].relative_to(root).as_posix()}"

    base_env = training_env_from_request(request_cfg, train_dataset, valid_dataset or None)
    pilot_env = base_env.copy()
    pilot_env.update(PILOT_OVERRIDES[request_cfg["method"]])
    pilot_env["OUTPUT_DIR"] = f"./outputs/pilot-{request_cfg['method']}"

    full_env = base_env.copy()
    full_env["OUTPUT_DIR"] = request_cfg["output_dir"]

    configs_dir = root / "configs"
    configs_dir.mkdir(parents=True, exist_ok=True)
    write_env_file(configs_dir / "pilot.env", pilot_env)
    write_env_file(configs_dir / "full.env", full_env)

    existing_remote = parse_env_file(configs_dir / "remote.auto.env")
    repo_base = sanitize_repo_base(request_cfg["model_id"], request_cfg["method"])
    remote_env = build_remote_env(existing_remote, repo_base, extra_asset_paths, root)
    write_env_file(configs_dir / "remote.auto.env", remote_env)
    gate_created = maybe_write_pai_gate(root, remote_env)

    plan_payload = {
        "request": request_cfg,
        "local_dataset_source": local_dataset_source,
        "remote_train_dataset": train_dataset,
        "remote_asset_paths": csv_items(remote_env["REMOTE_ASSET_PATHS"]),
        "local_dataset_bootstrap": None
        if dataset_bootstrap is None
        else {
            "train_path": str(dataset_bootstrap["train_path"]),
            "valid_path": str(dataset_bootstrap["valid_path"]),
        },
        "remote_env_file": str(configs_dir / "remote.auto.env"),
        "pilot_env_file": str(configs_dir / "pilot.env"),
        "full_env_file": str(configs_dir / "full.env"),
        "credential_gate_file": str(configs_dir / "pai.required.env") if gate_created else "",
    }
    write_json(configs_dir / "training.plan.json", plan_payload)

    result = {
        "workspace_root": str(root),
        "method": request_cfg["method"],
        "model_id": request_cfg["model_id"],
        "local_dataset_source": local_dataset_source,
        "train_dataset": train_dataset,
        "valid_dataset": valid_dataset,
        "remote_asset_paths": csv_items(remote_env["REMOTE_ASSET_PATHS"]),
        "credential_gate_created": gate_created,
        "changed_files": changed_files,
        "generated_files": [
            str(configs_dir / "training.plan.json"),
            str(configs_dir / "remote.auto.env"),
            str(configs_dir / "pilot.env"),
            str(configs_dir / "full.env"),
        ]
        + ([str(configs_dir / "pai.required.env")] if gate_created else []),
        "next_commands": [
            "uv run scripts/submit_pai_dlc.py --env-file configs/remote.auto.env --mode pilot --wait --retry-once",
            "uv run scripts/submit_pai_dlc.py --env-file configs/remote.auto.env --mode full --wait",
        ],
    }
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
