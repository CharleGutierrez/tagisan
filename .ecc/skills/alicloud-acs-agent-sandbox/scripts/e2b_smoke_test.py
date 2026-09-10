#!/usr/bin/env python3
"""Exercise ACS Agent Sandbox E2B APIs and always attempt cleanup."""

from __future__ import annotations

import argparse
import json
import os
import posixpath
import shlex
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


MARKER = "acs-agent-sandbox-smoke-ok"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Test E2B code, file, and command APIs, then kill the Sandbox"
    )
    parser.add_argument("--template", default="code-interpreter")
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument(
        "--private-protocol",
        action="store_true",
        help=(
            "patch the E2B SDK for the OpenKruise Agents private HTTP protocol; "
            "set E2B_DOMAIN to the local port-forward endpoint"
        ),
    )
    parser.add_argument("--storage-type", choices=("oss", "nas"))
    parser.add_argument("--storage-pv", help="Existing CSI PV to mount")
    parser.add_argument(
        "--storage-mount-path", help="Absolute empty directory inside the Sandbox"
    )
    parser.add_argument("--storage-sub-path", help="Optional PV subpath")
    parser.add_argument("--storage-read-only", action="store_true")
    parser.add_argument(
        "--credential-provider",
        help="CredentialProvider name; required for OSS Agent Identity",
    )
    parser.add_argument(
        "--agent-name", help="AgentIdentity name; required for OSS Agent Identity"
    )
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    storage_options = (
        args.storage_pv,
        args.storage_mount_path,
        args.storage_sub_path,
        args.storage_read_only,
        args.credential_provider,
        args.agent_name,
    )
    if any(storage_options) and not args.storage_type:
        parser.error("--storage-type is required when a storage option is used")
    if args.storage_type and (not args.storage_pv or not args.storage_mount_path):
        parser.error(
            "--storage-pv and --storage-mount-path are required with --storage-type"
        )
    if args.storage_mount_path and not args.storage_mount_path.startswith("/"):
        parser.error("--storage-mount-path must be absolute")
    if args.storage_type == "oss" and (
        not args.credential_provider or not args.agent_name
    ):
        parser.error(
            "OSS requires --credential-provider and --agent-name for Agent Identity"
        )
    if args.storage_type == "nas" and (
        args.credential_provider or args.agent_name
    ):
        parser.error("NAS does not use --credential-provider or --agent-name")
    return args


def build_metadata(args: argparse.Namespace) -> dict[str, str] | None:
    if not args.storage_type:
        return None

    volume: dict[str, Any] = {
        "pvName": args.storage_pv,
        "mountPath": args.storage_mount_path,
        "readOnly": args.storage_read_only,
    }
    if args.storage_sub_path:
        volume["subPath"] = args.storage_sub_path

    metadata = {
        "e2b.agents.kruise.io/csi-volume-config": json.dumps(
            [volume], separators=(",", ":")
        )
    }
    if args.storage_type == "oss":
        volume["attributes"] = {
            "credentialProviderName": args.credential_provider
        }
        metadata["e2b.agents.kruise.io/csi-volume-config"] = json.dumps(
            [volume], separators=(",", ":")
        )
        metadata["security.agents.kruise.io/agent-name"] = args.agent_name
    return metadata


def safe_error(exc: Exception) -> str:
    value = str(exc)
    for secret_name in ("E2B_API_KEY", "E2B_ACCESS_TOKEN"):
        secret = os.getenv(secret_name)
        if secret:
            value = value.replace(secret, "<redacted>")
    return value[:1000]


def main() -> int:
    args = parse_args()
    report: dict[str, Any] = {
        "schemaVersion": 1,
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "template": args.template,
        "transport": "private-http" if args.private_protocol else "native-https",
        "domainConfigured": bool(os.getenv("E2B_DOMAIN")),
        "apiKeyConfigured": bool(os.getenv("E2B_API_KEY")),
        "checks": {},
        "cleanup": {"attempted": False, "succeeded": False},
    }
    if args.storage_type:
        report["storage"] = {
            "type": args.storage_type,
            "pvName": args.storage_pv,
            "mountPath": args.storage_mount_path,
            "subPath": args.storage_sub_path,
            "readOnly": args.storage_read_only,
        }
    sandbox = None
    exit_code = 2

    try:
        if not report["domainConfigured"] or not report["apiKeyConfigured"]:
            raise RuntimeError("E2B_DOMAIN and E2B_API_KEY must be configured")

        if args.private_protocol:
            try:
                from kruise_agents.patch_e2b import patch_e2b
            except ImportError as exc:
                raise RuntimeError(
                    "install the OpenKruise Agents customized E2B package"
                ) from exc
            patch_e2b(https=False)

        from e2b_code_interpreter import Sandbox

        create_options: dict[str, Any] = {
            "template": args.template,
            "timeout": args.timeout,
        }
        metadata = build_metadata(args)
        if metadata:
            create_options["metadata"] = metadata
        sandbox = Sandbox.create(**create_options)
        report["sandboxId"] = sandbox.sandbox_id

        execution = sandbox.run_code(f"print('{MARKER}')")
        stdout = "".join(getattr(execution.logs, "stdout", []) or [])
        report["checks"]["runCode"] = MARKER in stdout

        sandbox.files.write("/tmp/acs-smoke.txt", MARKER)
        report["checks"]["fileWriteRead"] = (
            sandbox.files.read("/tmp/acs-smoke.txt") == MARKER
        )

        command = sandbox.commands.run("cat /tmp/acs-smoke.txt")
        report["checks"]["commandRun"] = (
            command.exit_code == 0 and command.stdout.strip() == MARKER
        )

        if args.storage_type:
            mount_path = shlex.quote(args.storage_mount_path)
            mount_check = sandbox.commands.run(
                f"test -d {mount_path} && test -r {mount_path} && "
                f"ls -A {mount_path} >/dev/null"
            )
            report["checks"]["storageReadable"] = mount_check.exit_code == 0

            if not args.storage_read_only:
                marker_name = f".acs-agent-sandbox-smoke-{uuid.uuid4().hex}"
                marker_path = shlex.quote(
                    posixpath.join(args.storage_mount_path, marker_name)
                )
                marker_value = shlex.quote(MARKER)
                storage_check = sandbox.commands.run(
                    "set -eu; "
                    f"printf %s {marker_value} > {marker_path}; "
                    f"test \"$(cat {marker_path})\" = {marker_value}; "
                    f"rm -f {marker_path}"
                )
                report["checks"]["storageWriteReadCleanup"] = (
                    storage_check.exit_code == 0
                )
        exit_code = 0 if all(report["checks"].values()) else 2
    except Exception as exc:
        report["error"] = safe_error(exc)
    finally:
        if sandbox is not None:
            report["cleanup"]["attempted"] = True
            try:
                report["cleanup"]["succeeded"] = bool(sandbox.kill())
            except Exception as exc:
                report["cleanup"]["error"] = safe_error(exc)
                exit_code = 2

        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(
            json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        print(f"Wrote sanitized E2B smoke report to {args.output}")

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
