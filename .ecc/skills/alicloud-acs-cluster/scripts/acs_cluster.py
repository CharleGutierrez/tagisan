#!/usr/bin/env python3
"""Create, inspect, connect to, and delete Alibaba Cloud ACS clusters."""

from __future__ import annotations

import argparse
import json
import os
import re
import stat
import sys
import time
from pathlib import Path
from typing import Any


def load_sdk() -> tuple[Any, Any, Any]:
    try:
        from alibabacloud_cs20151215.client import Client
        from alibabacloud_cs20151215 import models
        from alibabacloud_tea_openapi import models as open_api_models
    except ImportError as exc:
        raise RuntimeError(
            "Install alibabacloud-cs20151215 and alibabacloud-tea-openapi"
        ) from exc
    return Client, models, open_api_models


def credential() -> tuple[str, str, str | None]:
    access_key_id = (
        os.getenv("ALIBABACLOUD_ACCESS_KEY_ID")
        or os.getenv("ALIBABA_CLOUD_ACCESS_KEY_ID")
        or os.getenv("ALICLOUD_ACCESS_KEY_ID")
    )
    access_key_secret = (
        os.getenv("ALIBABACLOUD_ACCESS_KEY_SECRET")
        or os.getenv("ALIBABA_CLOUD_ACCESS_KEY_SECRET")
        or os.getenv("ALICLOUD_ACCESS_KEY_SECRET")
    )
    token = (
        os.getenv("ALIBABACLOUD_SECURITY_TOKEN")
        or os.getenv("ALIBABA_CLOUD_SECURITY_TOKEN")
        or os.getenv("ALICLOUD_SECURITY_TOKEN")
    )
    if not access_key_id or not access_key_secret:
        raise RuntimeError("Alibaba Cloud AccessKey environment variables are required")
    return access_key_id, access_key_secret, token


def safe_error(exc: Exception) -> str:
    value = str(exc)
    for name in (
        "ALIBABACLOUD_ACCESS_KEY_ID",
        "ALIBABACLOUD_ACCESS_KEY_SECRET",
        "ALIBABACLOUD_SECURITY_TOKEN",
        "ALIBABA_CLOUD_ACCESS_KEY_ID",
        "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
        "ALIBABA_CLOUD_SECURITY_TOKEN",
        "ALICLOUD_ACCESS_KEY_ID",
        "ALICLOUD_ACCESS_KEY_SECRET",
        "ALICLOUD_SECURITY_TOKEN",
    ):
        secret = os.getenv(name)
        if secret:
            value = value.replace(secret, "<redacted>")
    value = re.sub(
        r'(?i)(adminApiKey|apiKey|accessKeySecret|password|token)(["\s:=]+)[^,}\s]+',
        r'\1\2<redacted>',
        value,
    )
    return value[:2000]


def create_client(region: str) -> tuple[Any, Any]:
    client_class, models, open_api_models = load_sdk()
    access_key_id, access_key_secret, token = credential()
    config = open_api_models.Config(
        access_key_id=access_key_id,
        access_key_secret=access_key_secret,
        security_token=token,
        region_id=region,
        endpoint=f"cs.{region}.aliyuncs.com",
    )
    return client_class(config), models


def render(value: Any, output: Path | None = None) -> None:
    text = json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if output:
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(text, encoding="utf-8")
        print(f"Wrote sanitized result to {output}")
    else:
        sys.stdout.write(text)


def safe_cluster(body: Any) -> dict[str, Any]:
    return {
        "clusterId": body.cluster_id,
        "name": body.name,
        "regionId": body.region_id,
        "state": body.state,
        "profile": body.profile,
        "clusterType": body.cluster_type,
        "clusterSpec": body.cluster_spec,
        "currentVersion": body.current_version,
        "vpcId": body.vpc_id,
        "vswitchIds": body.vswitch_ids,
        "serviceCidr": body.service_cidr,
        "securityGroupId": body.security_group_id,
        "deletionProtection": body.deletion_protection,
        "created": body.created,
        "updated": body.updated,
    }


def create_shape(args: argparse.Namespace) -> dict[str, Any]:
    shape: dict[str, Any] = {
        "name": args.name,
        "region_id": args.region,
        "cluster_type": "ManagedKubernetes",
        "profile": "Acs",
        "cluster_spec": "ack.pro.small",
        "vpcid": args.vpc_id,
        "vswitch_ids": args.vswitch_id,
        "service_cidr": args.service_cidr,
        "snat_entry": args.snat,
        "endpoint_public_access": args.public_api,
        "is_enterprise_security_group": args.enterprise_security_group,
        "deletion_protection": args.deletion_protection,
        "ip_stack": "ipv4",
        "timezone": "Asia/Shanghai",
    }
    if args.kubernetes_version:
        shape["kubernetes_version"] = args.kubernetes_version
    if args.addon:
        shape["addons"] = [{"name": item, "config": ""} for item in args.addon]
    return shape


def command_create(args: argparse.Namespace) -> int:
    shape = create_shape(args)
    if args.dry_run:
        render({"dryRun": True, "request": shape}, args.output)
        return 0

    client, models = create_client(args.region)
    sdk_shape = dict(shape)
    sdk_shape["addons"] = [models.Addon(**item) for item in shape.get("addons", [])]
    response = client.create_cluster(models.CreateClusterRequest(**sdk_shape))
    result: dict[str, Any] = {
        "clusterId": response.body.cluster_id,
        "requestId": response.body.request_id,
        "taskId": response.body.task_id,
        "submitted": True,
    }
    if args.no_wait:
        render(result, args.output)
        return 0

    deadline = time.monotonic() + args.timeout
    while time.monotonic() < deadline:
        detail = client.describe_cluster_detail(response.body.cluster_id).body
        state = (detail.state or "").lower()
        if state == "running":
            result["cluster"] = safe_cluster(detail)
            render(result, args.output)
            return 0
        if state in {"failed", "error"}:
            result["cluster"] = safe_cluster(detail)
            render(result, args.output)
            return 2
        time.sleep(args.interval)

    result["waitTimedOutSeconds"] = args.timeout
    render(result, args.output)
    return 2


def command_describe(args: argparse.Namespace) -> int:
    client, _ = create_client(args.region)
    render(safe_cluster(client.describe_cluster_detail(args.cluster_id).body), args.output)
    return 0


def command_resources(args: argparse.Namespace) -> int:
    client, models = create_client(args.region)
    response = client.describe_cluster_resources(
        args.cluster_id,
        models.DescribeClusterResourcesRequest(
            with_addon_resources=args.with_addon_resources
        ),
    )
    resources = []
    for item in response.body or []:
        resources.append(
            {
                "instanceId": item.instance_id,
                "resourceType": item.resource_type,
                "state": item.state,
                "autoCreated": item.auto_create == 1,
                "creatorType": item.creator_type,
                "deleteBehavior": (
                    item.delete_behavior.to_map() if item.delete_behavior else None
                ),
            }
        )
    render(
        {"clusterId": args.cluster_id, "count": len(resources), "resources": resources},
        args.output,
    )
    return 0


def command_kubeconfig(args: argparse.Namespace) -> int:
    client, models = create_client(args.region)
    response = client.describe_cluster_user_kubeconfig(
        args.cluster_id,
        models.DescribeClusterUserKubeconfigRequest(
            private_ip_address=args.private,
            temporary_duration_minutes=args.temporary_minutes,
        ),
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(response.body.config, encoding="utf-8")
    args.output.chmod(stat.S_IRUSR | stat.S_IWUSR)
    print(
        f"Wrote protected KubeConfig to {args.output}; "
        f"expiration={response.body.expiration}"
    )
    return 0


def addon_config(path: Path | None) -> str:
    if path is None:
        return ""
    mode = path.stat().st_mode
    if mode & (stat.S_IRWXG | stat.S_IRWXO):
        raise RuntimeError("addon config file must not be accessible by group or others")
    return path.read_text(encoding="utf-8").strip()


def command_addon_status(args: argparse.Namespace) -> int:
    client, _ = create_client(args.region)
    body = client.describe_cluster_addon_instance(
        args.cluster_id, args.addon_name
    ).body
    logging = getattr(body, "logging", None)
    render(
        {
            "clusterId": args.cluster_id,
            "name": getattr(body, "name", args.addon_name),
            "state": getattr(body, "state", None),
            "version": getattr(body, "version", None),
            "logging": {
                "capable": getattr(logging, "capable", None),
                "enabled": getattr(logging, "enabled", None),
            }
            if logging
            else None,
            "configRedacted": True,
        },
        args.output,
    )
    return 0


def command_install_addon(args: argparse.Namespace) -> int:
    config = addon_config(args.config_file)
    preview = {
        "clusterId": args.cluster_id,
        "addon": args.addon_name,
        "version": args.version,
        "configProvided": bool(config),
        "configRedacted": True,
    }
    if args.dry_run:
        render({"dryRun": True, "request": preview}, args.output)
        return 0
    client, models = create_client(args.region)
    response = client.install_cluster_addons(
        args.cluster_id,
        models.InstallClusterAddonsRequest(
            body=[
                models.InstallClusterAddonsRequestBody(
                    name=args.addon_name, version=args.version, config=config
                )
            ]
        ),
    )
    render(
        {
            **preview,
            "submitted": True,
            "statusCode": response.status_code,
        },
        args.output,
    )
    return 0


def command_modify_addon(args: argparse.Namespace) -> int:
    config = addon_config(args.config_file)
    preview = {
        "clusterId": args.cluster_id,
        "addon": args.addon_name,
        "configProvided": bool(config),
        "configRedacted": True,
    }
    if args.dry_run:
        render({"dryRun": True, "request": preview}, args.output)
        return 0
    client, models = create_client(args.region)
    response = client.modify_cluster_addon(
        args.cluster_id,
        args.addon_name,
        models.ModifyClusterAddonRequest(config=config),
    )
    render(
        {
            **preview,
            "submitted": True,
            "statusCode": response.status_code,
        },
        args.output,
    )
    return 0


def command_uninstall_addon(args: argparse.Namespace) -> int:
    require_confirmation(args)
    if args.confirm_addon != args.addon_name:
        raise RuntimeError("--confirm-addon must exactly match --addon-name")
    preview = {
        "clusterId": args.cluster_id,
        "addon": args.addon_name,
        "cleanupCloudResources": args.cleanup_cloud_resources,
    }
    if args.dry_run:
        render({"dryRun": True, "request": preview}, args.output)
        return 0
    client, models = create_client(args.region)
    response = client.un_install_cluster_addons(
        args.cluster_id,
        models.UnInstallClusterAddonsRequest(
            addons=[
                models.UnInstallClusterAddonsRequestAddons(
                    name=args.addon_name,
                    cleanup_cloud_resources=args.cleanup_cloud_resources,
                )
            ]
        ),
    )
    render(
        {
            **preview,
            "submitted": True,
            "statusCode": response.status_code,
        },
        args.output,
    )
    return 0


def require_confirmation(args: argparse.Namespace) -> None:
    if args.confirm_cluster_id != args.cluster_id:
        raise RuntimeError("--confirm-cluster-id must exactly match --cluster-id")


def command_disable_protection(args: argparse.Namespace) -> int:
    require_confirmation(args)
    if args.dry_run:
        render(
            {
                "dryRun": True,
                "clusterId": args.cluster_id,
                "deletionProtection": False,
            },
            args.output,
        )
        return 0
    client, models = create_client(args.region)
    response = client.modify_cluster(
        args.cluster_id,
        models.ModifyClusterRequest(deletion_protection=False),
    )
    render(
        {
            "clusterId": args.cluster_id,
            "deletionProtection": False,
            "requestId": getattr(response.body, "request_id", None),
        },
        args.output,
    )
    return 0


def command_delete(args: argparse.Namespace) -> int:
    require_confirmation(args)
    delete_types = ["SLB", "ALB", "SLS_Data", "SLS_ControlPlane", "PrivateZone"]
    preview = {
        "clusterId": args.cluster_id,
        "retainAllResources": not args.delete_associated,
        "deleteResourceTypes": delete_types if args.delete_associated else [],
    }
    if args.dry_run:
        render({"dryRun": True, "request": preview}, args.output)
        return 0

    client, models = create_client(args.region)
    options = []
    if args.delete_associated:
        options = [
            models.DeleteClusterRequestDeleteOptions(
                resource_type=kind, delete_mode="delete"
            )
            for kind in delete_types
        ]
    response = client.delete_cluster(
        args.cluster_id,
        models.DeleteClusterRequest(
            delete_options=options,
            retain_all_resources=not args.delete_associated,
        ),
    )
    render(
        {
            **preview,
            "submitted": True,
            "requestId": getattr(response.body, "request_id", None),
            "taskId": getattr(response.body, "task_id", None),
        },
        args.output,
    )
    return 0


def add_region(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--region", required=True, help="region ID, e.g. cn-hangzhou")


def add_cluster(parser: argparse.ArgumentParser) -> None:
    add_region(parser)
    parser.add_argument("--cluster-id", required=True)


def add_json_output(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--output", type=Path, help="write sanitized JSON here")


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description="Manage Alibaba Cloud ACS clusters")
    commands = root.add_subparsers(dest="command", required=True)

    create = commands.add_parser("create", help="create an ACS cluster")
    add_region(create)
    create.add_argument("--name", required=True)
    create.add_argument("--vpc-id", required=True)
    create.add_argument("--vswitch-id", action="append", required=True)
    create.add_argument("--service-cidr", required=True)
    create.add_argument("--kubernetes-version")
    create.add_argument("--addon", action="append")
    create.add_argument("--snat", action="store_true")
    create.add_argument("--public-api", action="store_true")
    create.add_argument("--enterprise-security-group", action="store_true")
    create.add_argument("--deletion-protection", action="store_true")
    create.add_argument("--timeout", type=int, default=1800)
    create.add_argument("--interval", type=int, default=20)
    create.add_argument("--no-wait", action="store_true")
    create.add_argument("--dry-run", action="store_true")
    add_json_output(create)
    create.set_defaults(handler=command_create)

    describe = commands.add_parser("describe", help="show a safe cluster summary")
    add_cluster(describe)
    add_json_output(describe)
    describe.set_defaults(handler=command_describe)

    resources = commands.add_parser("resources", help="list associated cloud resources")
    add_cluster(resources)
    resources.add_argument("--with-addon-resources", action="store_true")
    add_json_output(resources)
    resources.set_defaults(handler=command_resources)

    kubeconfig = commands.add_parser("kubeconfig", help="write a protected KubeConfig")
    add_cluster(kubeconfig)
    kubeconfig.add_argument("--private", action="store_true")
    kubeconfig.add_argument("--temporary-minutes", type=int, default=60)
    kubeconfig.add_argument("--output", type=Path, required=True)
    kubeconfig.set_defaults(handler=command_kubeconfig)

    addon_status = commands.add_parser(
        "addon-status", help="show redacted addon instance status"
    )
    add_cluster(addon_status)
    addon_status.add_argument("--addon-name", required=True)
    add_json_output(addon_status)
    addon_status.set_defaults(handler=command_addon_status)

    install_addon = commands.add_parser("install-addon", help="install one addon")
    add_cluster(install_addon)
    install_addon.add_argument("--addon-name", required=True)
    install_addon.add_argument("--version")
    install_addon.add_argument("--config-file", type=Path)
    install_addon.add_argument("--dry-run", action="store_true")
    add_json_output(install_addon)
    install_addon.set_defaults(handler=command_install_addon)

    modify_addon = commands.add_parser("modify-addon", help="replace one addon config")
    add_cluster(modify_addon)
    modify_addon.add_argument("--addon-name", required=True)
    modify_addon.add_argument("--config-file", type=Path, required=True)
    modify_addon.add_argument("--dry-run", action="store_true")
    add_json_output(modify_addon)
    modify_addon.set_defaults(handler=command_modify_addon)

    uninstall_addon = commands.add_parser(
        "uninstall-addon", help="uninstall one explicitly confirmed addon"
    )
    add_cluster(uninstall_addon)
    uninstall_addon.add_argument("--addon-name", required=True)
    uninstall_addon.add_argument("--confirm-cluster-id", required=True)
    uninstall_addon.add_argument("--confirm-addon", required=True)
    uninstall_addon.add_argument("--cleanup-cloud-resources", action="store_true")
    uninstall_addon.add_argument("--dry-run", action="store_true")
    add_json_output(uninstall_addon)
    uninstall_addon.set_defaults(handler=command_uninstall_addon)

    disable = commands.add_parser(
        "disable-deletion-protection", help="disable cluster deletion protection"
    )
    add_cluster(disable)
    disable.add_argument("--confirm-cluster-id", required=True)
    disable.add_argument("--dry-run", action="store_true")
    add_json_output(disable)
    disable.set_defaults(handler=command_disable_protection)

    delete = commands.add_parser("delete", help="delete an ACS cluster")
    add_cluster(delete)
    delete.add_argument("--confirm-cluster-id", required=True)
    delete.add_argument("--delete-associated", action="store_true")
    delete.add_argument("--dry-run", action="store_true")
    add_json_output(delete)
    delete.set_defaults(handler=command_delete)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        return args.handler(args)
    except Exception as exc:
        print(f"error: {type(exc).__name__}: {safe_error(exc)}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
