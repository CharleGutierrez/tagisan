#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = [
#   "alibabacloud-pai-dlc20201203>=1.4.17",
#   "alibabacloud-tea-openapi>=0.3.15",
#   "modelscope>=1.16.0",
#   "packaging>=23.2",
# ]
# ///

"""Submit and monitor a PAI DLC job from local control plane.

This script is intentionally generic so multiple skills can reuse the same
submission path. It supports two execution patterns:

1. Provide `USER_COMMAND` directly in env files.
2. Provide ms-swift-style training variables and let the script build a
   `swift sft` or `swift rlhf --rlhf_type ...` command automatically.

If a remote job needs local files, set `REMOTE_ASSET_PATHS` and the script will
upload those paths to a temporary or user-chosen ModelScope dataset repo before
submission. The remote command then downloads that repo into `./remote_assets`.
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import sys
import time
from pathlib import Path
from typing import Any

from alibabacloud_pai_dlc20201203 import client as dlc_client
from alibabacloud_pai_dlc20201203 import models as dlc_models
from alibabacloud_tea_openapi import models as openapi_models
from modelscope.hub.api import HubApi

TERMINAL_STATUSES = {"Succeeded", "Failed", "Stopped"}
REDACT_KEYS = ("token", "secret", "password", "access_key")


def parse_env_file(path: Path) -> dict[str, str]:
    data: dict[str, str] = {}
    if not path.exists():
        raise FileNotFoundError(path)
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        data[key.strip()] = value.strip()
    return data


def merge_env_sources(env_file: Path, mode: str) -> dict[str, str]:
    merged = dict(os.environ)
    merged.update(parse_env_file(env_file))
    if mode:
        mode_file = env_file.with_name(f"{mode}.env")
        if mode_file.exists():
            merged.update(parse_env_file(mode_file))
    return merged


def workspace_root_from_env_file(env_file: Path) -> Path:
    parent = env_file.resolve().parent
    if parent.name == "configs":
        return parent.parent
    return parent


def csv_items(raw: str) -> list[str]:
    return [part.strip() for part in raw.split(",") if part.strip()]


def truthy(raw: str | None) -> bool:
    if raw is None:
        return False
    return raw.strip().lower() in {"1", "true", "yes", "on"}


def quote(value: str) -> str:
    return shlex.quote(str(value))


def redact_mapping(payload: Any) -> Any:
    if isinstance(payload, dict):
        out: dict[str, Any] = {}
        for key, value in payload.items():
            if any(part in key.lower() for part in REDACT_KEYS):
                out[key] = "***"
            else:
                out[key] = redact_mapping(value)
        return out
    if isinstance(payload, list):
        return [redact_mapping(item) for item in payload]
    return payload


def require_env(env: dict[str, str], key: str) -> str:
    value = env.get(key, "").strip()
    if not value:
        raise ValueError(f"Missing required environment value: {key}")
    return value


def choose_asset_repo(env: dict[str, str]) -> str:
    explicit = env.get("REMOTE_ASSET_REPO", "").strip()
    if explicit:
        return explicit
    owner = env.get("MS_REPO_OWNER", "").strip()
    base = env.get("MS_REPO_BASE", "").strip()
    if owner and base:
        return f"{owner}/{base}-assets"
    return ""


def choose_publish_repo(env: dict[str, str]) -> str:
    explicit = env.get("PUBLISH_REPO_ID", "").strip()
    if explicit:
        return explicit
    owner = env.get("MS_REPO_OWNER", "").strip()
    base = env.get("MS_REPO_BASE", "").strip()
    if owner and base:
        return f"{owner}/{base}"
    return ""


def path_in_repo(root: Path, item: Path) -> str:
    try:
        rel = item.resolve().relative_to(root.resolve())
        return rel.as_posix()
    except ValueError:
        return item.name


def upload_remote_assets(env: dict[str, str], root: Path) -> dict[str, Any]:
    repo_id = choose_asset_repo(env)
    raw_paths = csv_items(env.get("REMOTE_ASSET_PATHS", ""))
    if not repo_id or not raw_paths:
        return {"repo_id": repo_id, "uploaded": []}

    token = env.get("MODELSCOPE_API_TOKEN", "").strip()
    if not token:
        raise ValueError(
            "REMOTE_ASSET_PATHS is set, but MODELSCOPE_API_TOKEN is missing. "
            "A token is required to upload local assets for remote execution."
        )

    api = HubApi()
    api.create_repo(repo_id, repo_type="dataset", token=token, exist_ok=True)

    uploaded: list[dict[str, str]] = []
    for raw in raw_paths:
        candidate = Path(raw)
        item = candidate if candidate.is_absolute() else (root / candidate)
        item = item.resolve()
        if not item.exists():
            raise FileNotFoundError(f"Remote asset path does not exist: {item}")
        repo_path = path_in_repo(root, item)
        if item.is_dir():
            api.upload_folder(
                repo_id=repo_id,
                folder_path=str(item),
                path_in_repo=repo_path,
                repo_type="dataset",
                token=token,
                commit_message=f"Upload remote asset folder {repo_path}",
            )
        else:
            api.upload_file(
                path_or_fileobj=str(item),
                path_in_repo=repo_path,
                repo_id=repo_id,
                repo_type="dataset",
                token=token,
                commit_message=f"Upload remote asset file {repo_path}",
            )
        uploaded.append({"local_path": str(item), "path_in_repo": repo_path})
    return {"repo_id": repo_id, "uploaded": uploaded}


def plan_remote_assets(env: dict[str, str], root: Path) -> dict[str, Any]:
    repo_id = choose_asset_repo(env)
    planned: list[dict[str, str]] = []
    for raw in csv_items(env.get("REMOTE_ASSET_PATHS", "")):
        candidate = Path(raw)
        item = candidate if candidate.is_absolute() else (root / candidate)
        planned.append(
            {
                "local_path": str(item.resolve()),
                "path_in_repo": path_in_repo(root, item.resolve()) if item.exists() else item.name,
            }
        )
    return {"repo_id": repo_id, "uploaded": [], "planned": planned}


def build_swift_train_command(env: dict[str, str]) -> str:
    method = env.get("TRAINER_METHOD", env.get("SWIFT_METHOD", "sft")).strip().lower()
    if method not in {"sft", "dpo", "grpo"}:
        raise ValueError(f"Unsupported TRAINER_METHOD/SWIFT_METHOD: {method}")

    model_id = require_env(env, "MODEL_ID")
    dataset = env.get("TRAIN_DATASET", "").strip() or env.get("DATASET", "").strip()
    if not dataset:
        raise ValueError("Missing TRAIN_DATASET/DATASET for swift command generation")

    output_dir = env.get("OUTPUT_DIR", f"./outputs/{method}").strip()
    if method == "sft":
        cmd = [
            "swift",
            "sft",
            "--model",
            quote(model_id),
            "--dataset",
            quote(dataset),
            "--output_dir",
            quote(output_dir),
        ]
    else:
        cmd = [
            "swift",
            "rlhf",
            "--rlhf_type",
            method,
            "--model",
            quote(model_id),
            "--dataset",
            quote(dataset),
            "--output_dir",
            quote(output_dir),
        ]

    train_type = env.get("TRAIN_TYPE", "lora").strip()
    if train_type:
        cmd.extend(["--train_type", quote(train_type)])
    if train_type in {"lora", "qlora"} and env.get("LORA_RANK", "").strip():
        cmd.extend(["--lora_rank", env["LORA_RANK"].strip()])
    if env.get("MAX_LENGTH", "").strip():
        cmd.extend(["--max_length", env["MAX_LENGTH"].strip()])
    if env.get("BATCH_SIZE", "").strip():
        cmd.extend(["--per_device_train_batch_size", env["BATCH_SIZE"].strip()])
    if env.get("GRAD_ACC", "").strip():
        cmd.extend(["--gradient_accumulation_steps", env["GRAD_ACC"].strip()])
    if env.get("LEARNING_RATE", "").strip():
        cmd.extend(["--learning_rate", env["LEARNING_RATE"].strip()])
    if env.get("EPOCHS", "").strip():
        cmd.extend(["--num_train_epochs", env["EPOCHS"].strip()])
    if env.get("MAX_STEPS", "").strip():
        cmd.extend(["--max_steps", env["MAX_STEPS"].strip()])
    if env.get("SAVE_STEPS", "").strip():
        cmd.extend(["--save_steps", env["SAVE_STEPS"].strip()])
    if env.get("EVAL_STEPS", "").strip():
        cmd.extend(["--eval_steps", env["EVAL_STEPS"].strip()])
    if env.get("LOGGING_STEPS", "").strip():
        cmd.extend(["--logging_steps", env["LOGGING_STEPS"].strip()])
    if env.get("SYSTEM_PROMPT", "").strip():
        cmd.extend(["--system", quote(env["SYSTEM_PROMPT"].strip())])
    if method == "dpo":
        if env.get("BETA", "").strip():
            cmd.extend(["--beta", env["BETA"].strip()])
        if env.get("LOSS_TYPE", "").strip():
            cmd.extend(["--loss_type", quote(env["LOSS_TYPE"].strip())])
    if method == "grpo" and env.get("USE_VLLM", "").strip():
        cmd.extend(["--use_vllm", quote(env["USE_VLLM"].strip())])

    val_dataset = env.get("VALID_DATASET", env.get("VAL_DATASET", "")).strip()
    if val_dataset:
        cmd.extend(["--val_dataset", quote(val_dataset)])

    extra = env.get("EXTRA_SWIFT_ARGS", "").strip()
    rendered = " ".join(cmd)
    if extra:
        rendered = f"{rendered} {extra}"

    publish_repo = choose_publish_repo(env)
    if publish_repo and env.get("MODELSCOPE_API_TOKEN", "").strip():
        rendered = (
            f"{rendered} && "
            f"(modelscope create {quote(publish_repo)} --repo_type model --visibility public || true) && "
            f"modelscope upload {quote(publish_repo)} {quote(output_dir)} checkpoints --repo-type model"
        )
    return rendered


def build_generic_publish_tail(env: dict[str, str]) -> str:
    repo_id = choose_publish_repo(env)
    local_path = env.get("PUBLISH_LOCAL_PATH", "").strip()
    repo_type = env.get("PUBLISH_REPO_TYPE", "model").strip()
    path_in_repo = env.get("PUBLISH_PATH_IN_REPO", "").strip()
    if not repo_id or not local_path or not path_in_repo or not env.get("MODELSCOPE_API_TOKEN", "").strip():
        return ""
    return (
        f"(modelscope create {quote(repo_id)} --repo_type {quote(repo_type)} --visibility public || true) && "
        f"modelscope upload {quote(repo_id)} {quote(local_path)} {quote(path_in_repo)} --repo-type {quote(repo_type)}"
    )


def build_user_command(env: dict[str, str]) -> str:
    direct = env.get("USER_COMMAND", "").strip()
    if direct:
        publish_tail = build_generic_publish_tail(env)
        return f"{direct} && {publish_tail}" if publish_tail else direct
    if env.get("MODEL_ID", "").strip():
        return build_swift_train_command(env)
    raise ValueError("Missing USER_COMMAND and no MODEL_ID-based ms-swift configuration was found.")


def build_remote_command(env: dict[str, str]) -> str:
    pieces = ["set -e"]

    bootstrap = env.get("REMOTE_BOOTSTRAP_COMMAND", "").strip()
    asset_repo = choose_asset_repo(env)
    needs_modelscope = bool(asset_repo or choose_publish_repo(env))
    if bootstrap:
        pieces.append(bootstrap)
    elif env.get("MODEL_ID", "").strip():
        pieces.append(
            "python -m pip install -U pip setuptools wheel && "
            "python -m pip install -U \"modelscope>=1.16.0\" \"ms-swift\""
        )
    elif needs_modelscope:
        pieces.append(
            "python -m pip install -U pip setuptools wheel && "
            "python -m pip install -U \"modelscope>=1.16.0\""
        )

    if asset_repo:
        pieces.append(f"modelscope download --dataset {quote(asset_repo)} --local_dir ./remote_assets")

    pieces.append(build_user_command(env))
    return " && ".join(pieces)


def build_remote_envs(env: dict[str, str]) -> dict[str, str]:
    result: dict[str, str] = {}
    passthrough = {
        "MODELSCOPE_API_TOKEN",
        "REMOTE_ASSET_REPO",
        "PUBLISH_REPO_ID",
        "PUBLISH_LOCAL_PATH",
        "PUBLISH_PATH_IN_REPO",
        "PUBLISH_REPO_TYPE",
        "MS_REPO_OWNER",
        "MS_REPO_BASE",
    }
    for key in passthrough:
        value = env.get(key, "").strip()
        if value:
            result[key] = value
    for key, value in env.items():
        if key.startswith("REMOTE_ENV_") and value.strip():
            result[key.removeprefix("REMOTE_ENV_")] = value.strip()
    return result


def create_client(env: dict[str, str]) -> dlc_client.Client:
    region = require_env(env, "PAI_REGION")
    config = openapi_models.Config(
        access_key_id=require_env(env, "ALIBABA_CLOUD_ACCESS_KEY_ID"),
        access_key_secret=require_env(env, "ALIBABA_CLOUD_ACCESS_KEY_SECRET"),
        security_token=env.get("ALIBABA_CLOUD_SECURITY_TOKEN", "").strip() or None,
        region_id=region,
        endpoint=f"pai-dlc.{region}.aliyuncs.com",
    )
    return dlc_client.Client(config)


def build_request(env: dict[str, str], user_command: str) -> dlc_models.CreateJobRequest:
    req = dlc_models.CreateJobRequest()
    req.display_name = env.get("PAI_DLC_DISPLAY_NAME", f"modelscope-{int(time.time())}")
    req.job_type = env.get("PAI_DLC_JOB_TYPE", "PyTorchJob")
    req.user_command = user_command
    workspace_id = env.get("PAI_WORKSPACE_ID", "").strip()
    if workspace_id:
        req.workspace_id = workspace_id
    if env.get("PAI_RESOURCE_ID", "").strip():
        req.resource_id = env["PAI_RESOURCE_ID"].strip()
    if env.get("PAI_DLC_MAX_MINUTES", "").strip():
        req.job_max_running_time_minutes = int(env["PAI_DLC_MAX_MINUTES"].strip())

    remote_envs = build_remote_envs(env)
    if remote_envs:
        req.envs = remote_envs

    spec = dlc_models.JobSpec()
    spec.type = env.get("PAI_DLC_ROLE_TYPE", "Worker")
    spec.pod_count = int(env.get("PAI_DLC_POD_COUNT", "1").strip())
    spec.image = env.get("PAI_DLC_IMAGE", "pytorch/pytorch:latest").strip()
    ecs_spec = env.get("PAI_DLC_ECS_SPEC", "").strip()
    if ecs_spec:
        spec.ecs_spec = ecs_spec
    else:
        resource = dlc_models.ResourceConfig()
        resource.cpu = env.get("PAI_DLC_CPU", "4").strip()
        resource.memory = env.get("PAI_DLC_MEMORY", "16Gi").strip()
        resource.gpu = env.get("PAI_DLC_GPU", "0").strip()
        if env.get("PAI_DLC_SHARED_MEMORY", "").strip():
            resource.shared_memory = env["PAI_DLC_SHARED_MEMORY"].strip()
        spec.resource_config = resource
    req.job_specs = [spec]
    return req


def get_job_status(client: dlc_client.Client, job_id: str) -> dict[str, Any]:
    req = dlc_models.GetJobRequest()
    req.need_detail = True
    body = client.get_job(job_id, req).body
    return {
        "job_id": getattr(body, "job_id", "") or getattr(body, "jobId", "") or job_id,
        "status": getattr(body, "status", ""),
        "reason_code": getattr(body, "reason_code", ""),
        "reason_message": getattr(body, "reason_message", ""),
        "workspace_id": getattr(body, "workspace_id", ""),
        "workspace_name": getattr(body, "workspace_name", ""),
    }


def submit_once(client: dlc_client.Client, request: dlc_models.CreateJobRequest) -> str:
    response = client.create_job(request)
    body = response.body
    job_id = getattr(body, "job_id", "") or getattr(body, "jobId", "")
    if not job_id:
        raise RuntimeError(f"CreateJob returned no job id: {response}")
    return job_id


def wait_for_terminal(
    client: dlc_client.Client, job_id: str, poll_interval: int
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    history: list[dict[str, Any]] = []
    while True:
        current = get_job_status(client, job_id)
        history.append(current)
        if current["status"] in TERMINAL_STATUSES:
            return current, history
        time.sleep(poll_interval)


def save_report(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Submit a PAI DLC job from env files")
    parser.add_argument("--env-file", required=True, help="Base env file, usually configs/remote.auto.env")
    parser.add_argument("--mode", default="", help="Optional mode env file, for example pilot or full")
    parser.add_argument("--wait", action="store_true", help="Poll job status until terminal state")
    parser.add_argument("--poll-interval", type=int, default=10, help="Polling interval in seconds")
    parser.add_argument("--retry-once", action="store_true", help="Retry one time if the job ends in Failed/Stopped")
    parser.add_argument("--dry-run", action="store_true", help="Print request payload without submitting")
    parser.add_argument("--report-file", default="", help="Optional JSON report output path")
    args = parser.parse_args()

    env_file = Path(args.env_file).resolve()
    root = workspace_root_from_env_file(env_file)
    env = merge_env_sources(env_file, args.mode)
    if args.dry_run:
        asset_upload = plan_remote_assets(env, root)
    else:
        asset_upload = upload_remote_assets(env, root)
    if asset_upload.get("repo_id"):
        env["REMOTE_ASSET_REPO"] = asset_upload["repo_id"]

    user_command = build_remote_command(env)
    request = build_request(env, user_command)
    request_map = redact_mapping(request.to_map())

    if args.dry_run:
        payload = {
            "mode": args.mode,
            "workspace_root": str(root),
            "asset_upload": redact_mapping(asset_upload),
            "request": request_map,
            "user_command": user_command,
        }
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return

    client = create_client(env)
    job_ids: list[str] = []
    terminal: dict[str, Any] | None = None
    history: list[dict[str, Any]] = []

    for attempt in range(2 if args.retry_once else 1):
        job_id = submit_once(client, request)
        job_ids.append(job_id)
        if not args.wait:
            terminal = {"job_id": job_id, "status": "Submitted"}
            break
        terminal, run_history = wait_for_terminal(client, job_id, args.poll_interval)
        history.extend(run_history)
        if terminal["status"] == "Succeeded" or attempt == 1 or not args.retry_once:
            break

    report = {
        "mode": args.mode,
        "workspace_root": str(root),
        "asset_upload": redact_mapping(asset_upload),
        "job_ids": job_ids,
        "terminal": terminal,
        "history": history,
        "request": request_map,
        "user_command": user_command,
    }

    if args.report_file:
        save_report(Path(args.report_file).resolve(), report)
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:  # noqa: BLE001
        print(json.dumps({"error": str(exc)}, ensure_ascii=False, indent=2), file=sys.stderr)
        raise
