#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Local run tracking CLI for ModelScope-oriented workflows.

The tracker stores run metadata, metrics, and artifact records in a project-local
folder so results can be inspected, summarized, and uploaded to ModelScope repos.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import shutil
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

DEFAULT_ROOT = ".modelscope-tracking"


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def parse_tags(raw: str) -> list[str]:
    if not raw.strip():
        return []
    return [part.strip() for part in raw.split(",") if part.strip()]


def load_json(path: Path, default: dict[str, Any]) -> dict[str, Any]:
    if not path.exists():
        return default
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def append_jsonl(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(payload, ensure_ascii=False) + "\n")


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    rows: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def tracking_root(root_arg: str) -> Path:
    return Path(root_arg).expanduser().resolve()


def run_dir(root: Path, run_id: str) -> Path:
    return root / "runs" / run_id


def run_manifest_path(root: Path, run_id: str) -> Path:
    return run_dir(root, run_id) / "run.json"


def require_manifest(root: Path, run_id: str) -> tuple[Path, dict[str, Any]]:
    manifest_path = run_manifest_path(root, run_id)
    if not manifest_path.exists():
        raise FileNotFoundError(f"Run not found: {run_id}")
    return manifest_path, json.loads(manifest_path.read_text(encoding="utf-8"))


def command_init(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    root.mkdir(parents=True, exist_ok=True)
    (root / "runs").mkdir(exist_ok=True)
    (root / "artifacts").mkdir(exist_ok=True)
    (root / "reports").mkdir(exist_ok=True)

    project = {
        "project": args.project,
        "created_at": utc_now(),
        "owner": args.owner,
        "description": args.description,
    }
    write_json(root / "project.json", project)
    print(f"Initialized tracking workspace at {root}")


def command_create_run(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    if not (root / "project.json").exists():
        raise FileNotFoundError(
            f"Tracking workspace is not initialized at {root}. Run: run_tracker.py init"
        )

    run_id = args.run_id or f"run-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}-{uuid.uuid4().hex[:8]}"
    path = run_manifest_path(root, run_id)
    if path.exists():
        raise ValueError(f"Run already exists: {run_id}")

    params = json.loads(args.params_json) if args.params_json else {}
    if not isinstance(params, dict):
        raise ValueError("--params-json must be a JSON object")

    manifest = {
        "run_id": run_id,
        "status": "running",
        "created_at": utc_now(),
        "updated_at": utc_now(),
        "model_id": args.model_id,
        "dataset_id": args.dataset_id,
        "task": args.task,
        "tags": parse_tags(args.tags),
        "params": params,
        "notes": args.notes,
    }

    path.parent.mkdir(parents=True, exist_ok=True)
    write_json(path, manifest)
    print(run_id)


def command_log_metric(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    manifest_path, manifest = require_manifest(root, args.run_id)

    metric = {
        "timestamp": args.timestamp or utc_now(),
        "step": args.step,
        "split": args.split,
        "name": args.name,
        "value": args.value,
    }
    append_jsonl(manifest_path.parent / "metrics.jsonl", metric)

    manifest["updated_at"] = utc_now()
    write_json(manifest_path, manifest)
    print(f"Logged metric {args.name} for {args.run_id}")


def command_add_artifact(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    manifest_path, manifest = require_manifest(root, args.run_id)

    source = Path(args.path).expanduser().resolve()
    if not source.exists():
        raise FileNotFoundError(f"Artifact path not found: {source}")

    stored_path = source
    if args.copy:
        target_dir = root / "artifacts" / args.run_id
        target_dir.mkdir(parents=True, exist_ok=True)
        stored_path = target_dir / source.name
        shutil.copy2(source, stored_path)

    digest = hashlib.sha256(source.read_bytes()).hexdigest() if source.is_file() else ""
    record = {
        "timestamp": utc_now(),
        "kind": args.kind,
        "label": args.label,
        "source_path": str(source),
        "path": str(stored_path),
        "sha256": digest,
        "size_bytes": source.stat().st_size if source.is_file() else 0,
    }
    append_jsonl(manifest_path.parent / "artifacts.jsonl", record)

    manifest["updated_at"] = utc_now()
    write_json(manifest_path, manifest)
    print(f"Registered artifact for {args.run_id}")


def command_complete_run(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    manifest_path, manifest = require_manifest(root, args.run_id)

    summary = json.loads(args.summary_json) if args.summary_json else {}
    if not isinstance(summary, dict):
        raise ValueError("--summary-json must be a JSON object")

    manifest["status"] = args.status
    manifest["completed_at"] = utc_now()
    manifest["updated_at"] = utc_now()
    if summary:
        manifest["summary"] = summary

    write_json(manifest_path, manifest)
    print(f"Completed {args.run_id} with status={args.status}")


def collect_runs(root: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    runs_root = root / "runs"
    if not runs_root.exists():
        return rows

    for path in sorted(runs_root.glob("*/run.json")):
        manifest = json.loads(path.read_text(encoding="utf-8"))
        metrics = read_jsonl(path.parent / "metrics.jsonl")
        artifacts = read_jsonl(path.parent / "artifacts.jsonl")
        metric_names = sorted({m.get("name", "") for m in metrics if m.get("name")})
        rows.append(
            {
                "run_id": manifest.get("run_id", path.parent.name),
                "status": manifest.get("status", "unknown"),
                "task": manifest.get("task", ""),
                "model_id": manifest.get("model_id", ""),
                "dataset_id": manifest.get("dataset_id", ""),
                "created_at": manifest.get("created_at", ""),
                "updated_at": manifest.get("updated_at", ""),
                "metrics_count": len(metrics),
                "artifact_count": len(artifacts),
                "metric_names": metric_names,
            }
        )
    return rows


def command_list_runs(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    runs = collect_runs(root)

    if args.format == "json":
        print(json.dumps(runs, indent=2, ensure_ascii=False))
        return

    if not runs:
        print("No runs found.")
        return

    header = f"{'run_id':<32} {'status':<10} {'metrics':<7} {'artifacts':<9} {'task'}"
    print(header)
    print("-" * len(header))
    for run in runs:
        print(
            f"{run['run_id']:<32} {run['status']:<10} {run['metrics_count']:<7} "
            f"{run['artifact_count']:<9} {run['task']}"
        )


def build_markdown_summary(root: Path, runs: list[dict[str, Any]]) -> str:
    project = load_json(root / "project.json", default={})
    lines = [
        f"# Tracking Summary: {project.get('project', 'unnamed-project')}",
        "",
        f"Generated at: {utc_now()}",
        "",
        "| Run ID | Status | Task | Metrics | Artifacts |",
        "|--------|--------|------|---------|-----------|",
    ]
    for run in runs:
        lines.append(
            f"| {run['run_id']} | {run['status']} | {run['task']} | "
            f"{run['metrics_count']} | {run['artifact_count']} |"
        )
    return "\n".join(lines) + "\n"


def command_summarize(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    runs = collect_runs(root)

    if args.format == "json":
        payload = {
            "generated_at": utc_now(),
            "runs": runs,
            "total_runs": len(runs),
            "status_breakdown": {
                status: sum(1 for r in runs if r["status"] == status)
                for status in sorted({r["status"] for r in runs})
            },
        }
        content = json.dumps(payload, indent=2, ensure_ascii=False) + "\n"
    else:
        content = build_markdown_summary(root, runs)

    if args.output:
        out = Path(args.output).expanduser().resolve()
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(content, encoding="utf-8")
        print(f"Wrote summary: {out}")
    else:
        print(content, end="")


def command_export_metrics_csv(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    run_path = run_dir(root, args.run_id)
    metrics = read_jsonl(run_path / "metrics.jsonl")
    if not metrics:
        raise ValueError(f"No metrics found for run: {args.run_id}")

    out = Path(args.output).expanduser().resolve()
    out.parent.mkdir(parents=True, exist_ok=True)

    fields = ["timestamp", "step", "split", "name", "value"]
    with out.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        for row in metrics:
            writer.writerow({k: row.get(k, "") for k in fields})

    print(f"Wrote CSV: {out}")


def command_validate(args: argparse.Namespace) -> None:
    root = tracking_root(args.root)
    errors: list[str] = []

    project_path = root / "project.json"
    if not project_path.exists():
        errors.append("Missing project.json")

    runs_root = root / "runs"
    if runs_root.exists():
        for manifest in runs_root.glob("*/run.json"):
            try:
                json.loads(manifest.read_text(encoding="utf-8"))
            except Exception as exc:
                errors.append(f"Invalid JSON: {manifest} ({exc})")

    if errors:
        for err in errors:
            print(err)
        raise SystemExit(1)

    print("Tracking workspace is valid.")


def add_root_argument(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--root", default=DEFAULT_ROOT, help="Tracking workspace root directory")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Local experiment tracker for ModelScope workflows")
    sub = parser.add_subparsers(dest="command", required=True)

    p_init = sub.add_parser("init", help="Initialize a tracking workspace")
    add_root_argument(p_init)
    p_init.add_argument("--project", required=True, help="Project name")
    p_init.add_argument("--owner", default="", help="Owner or team")
    p_init.add_argument("--description", default="", help="Project description")
    p_init.set_defaults(func=command_init)

    p_create = sub.add_parser("create-run", help="Create a run manifest")
    add_root_argument(p_create)
    p_create.add_argument("--run-id", default="", help="Optional explicit run id")
    p_create.add_argument("--model-id", default="", help="Model identifier")
    p_create.add_argument("--dataset-id", default="", help="Dataset identifier")
    p_create.add_argument("--task", default="", help="Task label (e.g., sft, eval, inference)")
    p_create.add_argument("--tags", default="", help="Comma-separated tags")
    p_create.add_argument("--params-json", default="{}", help="JSON object for hyperparameters/config")
    p_create.add_argument("--notes", default="", help="Optional notes")
    p_create.set_defaults(func=command_create_run)

    p_metric = sub.add_parser("log-metric", help="Append a metric record to a run")
    add_root_argument(p_metric)
    p_metric.add_argument("--run-id", required=True)
    p_metric.add_argument("--name", required=True, help="Metric name")
    p_metric.add_argument("--value", type=float, required=True, help="Metric value")
    p_metric.add_argument("--step", type=int, default=0)
    p_metric.add_argument("--split", default="train", help="Data split")
    p_metric.add_argument("--timestamp", default="", help="Optional ISO timestamp override")
    p_metric.set_defaults(func=command_log_metric)

    p_artifact = sub.add_parser("add-artifact", help="Register an artifact for a run")
    add_root_argument(p_artifact)
    p_artifact.add_argument("--run-id", required=True)
    p_artifact.add_argument("--path", required=True, help="Local artifact path")
    p_artifact.add_argument("--kind", default="file", help="Artifact kind (checkpoint/report/eval/file)")
    p_artifact.add_argument("--label", default="", help="Human-readable artifact label")
    p_artifact.add_argument("--copy", action="store_true", help="Copy artifact into tracking workspace")
    p_artifact.set_defaults(func=command_add_artifact)

    p_complete = sub.add_parser("complete-run", help="Mark a run as completed")
    add_root_argument(p_complete)
    p_complete.add_argument("--run-id", required=True)
    p_complete.add_argument("--status", choices=["succeeded", "failed", "canceled"], default="succeeded")
    p_complete.add_argument("--summary-json", default="{}", help="JSON object with final summary fields")
    p_complete.set_defaults(func=command_complete_run)

    p_list = sub.add_parser("list-runs", help="List tracked runs")
    add_root_argument(p_list)
    p_list.add_argument("--format", choices=["table", "json"], default="table")
    p_list.set_defaults(func=command_list_runs)

    p_summary = sub.add_parser("summarize", help="Generate project summary")
    add_root_argument(p_summary)
    p_summary.add_argument("--format", choices=["markdown", "json"], default="markdown")
    p_summary.add_argument("--output", default="", help="Optional output path")
    p_summary.set_defaults(func=command_summarize)

    p_export = sub.add_parser("export-metrics-csv", help="Export one run's metrics to CSV")
    add_root_argument(p_export)
    p_export.add_argument("--run-id", required=True)
    p_export.add_argument("--output", required=True)
    p_export.set_defaults(func=command_export_metrics_csv)

    p_validate = sub.add_parser("validate", help="Validate tracking workspace structure")
    add_root_argument(p_validate)
    p_validate.set_defaults(func=command_validate)

    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
