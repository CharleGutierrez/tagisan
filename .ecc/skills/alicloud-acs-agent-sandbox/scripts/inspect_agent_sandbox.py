#!/usr/bin/env python3
"""Read-only ACS Agent Sandbox inspection with redacted JSON output."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


NAMESPACED_RESOURCES = [
    "sandboxes.agents.kruise.io",
    "sandboxsets.agents.kruise.io",
    "sandboxclaims.agents.kruise.io",
    "checkpoints.agents.kruise.io",
    "sandboxupdateops.agents.kruise.io",
    "containerrecreaterequests.apps.kruise.io",
    "trafficpolicies.network.alibabacloud.com",
    "securityprofiles.agents.kruise.io",
    "agentidentities.agentidentity.alibabacloud.com",
    "credentialproviders.agentidentity.alibabacloud.com",
    "agentroles.agentidentity.alibabacloud.com",
    "agentrolebindings.agentidentity.alibabacloud.com",
]

CLUSTER_RESOURCES = [
    "globaltrafficpolicies.network.alibabacloud.com",
    "globalsecurityprofiles.agents.kruise.io",
    "persistentvolumes",
]

STORAGE_DRIVERS = {
    "ossplugin.csi.alibabacloud.com": "oss",
    "nasplugin.csi.alibabacloud.com": "nas",
}

SAFE_LABELS = {
    "agents.kruise.io/sandbox-claimed",
    "agents.kruise.io/sandbox-pool",
    "agents.kruise.io/sandbox-template",
    "agents.kruise.io/claim-name",
    "alibabacloud.com/acs",
    "alibabacloud.com/compute-class",
    "alibabacloud.com/compute-qos",
    "security.agents.kruise.io/agent-name",
}

SAFE_ANNOTATIONS = {
    "network.alibabacloud.com/vswitch-ids",
    "network.alibabacloud.com/security-group-ids",
    "network.alibabacloud.com/network-policy-mode",
    "network.alibabacloud.com/enable-network-policy-agent",
    "security.agents.kruise.io/enable-jwt-auth",
    "ops.alibabacloud.com/pause-enabled",
    "scaling.alibabacloud.com/enable-inplace-resource-resize",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Inspect ACS Agent Sandbox resources without reading Secret contents."
    )
    parser.add_argument("--kubectl", default="kubectl", help="kubectl executable")
    parser.add_argument("--context", help="kubectl context")
    parser.add_argument("--kubeconfig", help="kubeconfig path")
    parser.add_argument(
        "--namespace",
        help="inspect one namespace instead of all namespaces",
    )
    parser.add_argument("--timeout", type=int, default=20, help="command timeout in seconds")
    parser.add_argument("--no-events", action="store_true", help="skip warning events")
    parser.add_argument("--output", type=Path, help="write JSON report to this path")
    return parser.parse_args()


class Kubectl:
    def __init__(self, args: argparse.Namespace) -> None:
        self.base = [args.kubectl]
        if args.context:
            self.base.extend(["--context", args.context])
        if args.kubeconfig:
            self.base.extend(["--kubeconfig", args.kubeconfig])
        self.timeout = args.timeout

    def run(self, command: list[str]) -> dict[str, Any]:
        full = [*self.base, *command]
        try:
            proc = subprocess.run(
                full,
                capture_output=True,
                text=True,
                timeout=self.timeout,
                check=False,
            )
        except FileNotFoundError:
            return {"ok": False, "error": f"executable not found: {self.base[0]}"}
        except subprocess.TimeoutExpired:
            return {"ok": False, "error": f"timed out after {self.timeout}s"}

        if proc.returncode != 0:
            return {
                "ok": False,
                "error": sanitize_error(proc.stderr.strip() or proc.stdout.strip()),
            }
        return {"ok": True, "stdout": proc.stdout}

    def json(self, command: list[str]) -> dict[str, Any]:
        result = self.run(command)
        if not result["ok"]:
            return result
        try:
            return {"ok": True, "data": json.loads(result["stdout"] or "{}")}
        except json.JSONDecodeError as exc:
            return {"ok": False, "error": f"invalid JSON from kubectl: {exc}"}


def sanitize_error(value: str) -> str:
    lines = value.splitlines()
    return "\n".join(lines[:8])[:2000]


def safe_metadata(item: dict[str, Any]) -> dict[str, Any]:
    meta = item.get("metadata", {})
    labels = meta.get("labels") or {}
    annotations = meta.get("annotations") or {}
    return {
        "name": meta.get("name"),
        "namespace": meta.get("namespace"),
        "created": meta.get("creationTimestamp"),
        "labels": {key: labels[key] for key in SAFE_LABELS if key in labels},
        "annotations": {
            key: annotations[key] for key in SAFE_ANNOTATIONS if key in annotations
        },
    }


def condition_map(status: dict[str, Any]) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for condition in status.get("conditions") or []:
        kind = condition.get("type")
        if kind:
            result[kind] = {
                "status": condition.get("status"),
                "reason": condition.get("reason"),
                "message": condition.get("message"),
                "lastTransitionTime": condition.get("lastTransitionTime"),
            }
    return result


def sandbox_storage_pvs(item: dict[str, Any]) -> list[str]:
    annotations = (item.get("metadata") or {}).get("annotations") or {}
    value = annotations.get("e2b.agents.kruise.io/csi-volume-config")
    if not value:
        return []
    try:
        volumes = json.loads(value)
    except (json.JSONDecodeError, TypeError):
        return []
    return sorted(
        {
            volume.get("pvName")
            for volume in volumes
            if isinstance(volume, dict) and volume.get("pvName")
        }
    )


def summarize_resource(resource: str, item: dict[str, Any]) -> dict[str, Any]:
    summary = safe_metadata(item)
    spec = item.get("spec") or {}
    status = item.get("status") or {}
    summary["generation"] = item.get("metadata", {}).get("generation")
    summary["observedGeneration"] = status.get("observedGeneration")

    if resource.startswith("sandboxes."):
        summary.update(
            {
                "phase": status.get("phase"),
                "message": status.get("message"),
                "nodeName": status.get("nodeName"),
                "shutdownTime": spec.get("shutdownTime"),
                "pauseTime": spec.get("pauseTime"),
                "paused": spec.get("paused", False),
                "storagePVs": sandbox_storage_pvs(item),
                "conditions": condition_map(status),
            }
        )
    elif resource.startswith("sandboxsets."):
        summary.update(
            {
                "desiredReplicas": spec.get("replicas"),
                "replicas": status.get("replicas"),
                "availableReplicas": status.get("availableReplicas"),
                "updatedReplicas": status.get("updatedReplicas"),
                "updatedAvailableReplicas": status.get("updatedAvailableReplicas"),
                "updateRevision": status.get("updateRevision"),
                "runtimes": [
                    runtime.get("name")
                    for runtime in spec.get("runtimes") or []
                    if runtime.get("name")
                ],
            }
        )
    elif resource.startswith("sandboxclaims."):
        summary.update(
            {
                "phase": status.get("phase"),
                "templateName": spec.get("templateName"),
                "desired": spec.get("replicas"),
                "claimed": status.get("claimedReplicas", status.get("claimed")),
                "message": status.get("message"),
            }
        )
    elif resource.startswith("checkpoints."):
        summary.update(
            {
                "phase": status.get("phase"),
                "podName": spec.get("podName"),
                "message": status.get("message"),
            }
        )
    elif resource.startswith("sandboxupdateops."):
        summary.update(
            {
                "phase": status.get("phase"),
                "total": status.get("total"),
                "updated": status.get("updated"),
                "updating": status.get("updating"),
                "failed": status.get("failed"),
                "message": status.get("message"),
            }
        )
    elif resource.startswith("containerrecreaterequests."):
        summary.update(
            {
                "phase": status.get("phase"),
                "podName": spec.get("podName"),
                "containers": [c.get("name") for c in spec.get("containers") or []],
                "states": status.get("containerRecreateStates") or [],
            }
        )
    elif resource == "persistentvolumes":
        csi = spec.get("csi") or {}
        attributes = csi.get("volumeAttributes") or {}
        driver = csi.get("driver")
        summary.update(
            {
                "phase": status.get("phase"),
                "capacity": (spec.get("capacity") or {}).get("storage"),
                "accessModes": spec.get("accessModes") or [],
                "reclaimPolicy": spec.get("persistentVolumeReclaimPolicy"),
                "storageClassName": spec.get("storageClassName"),
                "csiDriver": driver,
                "storageType": STORAGE_DRIVERS.get(driver),
                "volumeHandle": csi.get("volumeHandle"),
                "authType": (
                    attributes.get("authType")
                    if driver == "ossplugin.csi.alibabacloud.com"
                    else None
                ),
                "mountOptions": spec.get("mountOptions") or [],
            }
        )
    else:
        summary["kind"] = item.get("kind")
    return summary


def summarize_pods(data: dict[str, Any]) -> list[dict[str, Any]]:
    pods = []
    for item in data.get("items") or []:
        status = item.get("status") or {}
        container_statuses = status.get("containerStatuses") or []
        pods.append(
            {
                **safe_metadata(item),
                "phase": status.get("phase"),
                "podIP": status.get("podIP"),
                "nodeName": status.get("nodeName"),
                "containers": [
                    {
                        "name": c.get("name"),
                        "image": c.get("image"),
                        "ready": c.get("ready"),
                        "restartCount": c.get("restartCount"),
                    }
                    for c in container_statuses
                ],
            }
        )
    return pods


def add_finding(
    findings: list[dict[str, str]], severity: str, code: str, message: str
) -> None:
    findings.append({"severity": severity, "code": code, "message": message})


def evaluate(report: dict[str, Any]) -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    if report.get("errors"):
        add_finding(
            findings,
            "high",
            "inspection_incomplete",
            f"Inspection has {len(report['errors'])} command errors; do not infer cluster health",
        )
    resources = report.get("resources", {})
    sandboxes = resources.get("sandboxes.agents.kruise.io", {}).get("items", [])
    sets = resources.get("sandboxsets.agents.kruise.io", {}).get("items", [])
    claims = resources.get("sandboxclaims.agents.kruise.io", {}).get("items", [])
    updates = resources.get("sandboxupdateops.agents.kruise.io", {}).get("items", [])
    pvs = resources.get("persistentvolumes", {}).get("items", [])

    for item in sandboxes:
        identity = f"{item.get('namespace')}/{item.get('name')}"
        if item.get("phase") == "Failed":
            add_finding(findings, "high", "sandbox_failed", f"Sandbox {identity} is Failed")
        ready = item.get("conditions", {}).get("Ready", {}).get("status")
        if item.get("phase") == "Running" and ready not in (None, "True"):
            add_finding(findings, "high", "sandbox_not_ready", f"Running Sandbox {identity} is not Ready")

    for item in sets:
        desired = item.get("desiredReplicas") or 0
        available = item.get("availableReplicas") or 0
        if available < desired:
            identity = f"{item.get('namespace')}/{item.get('name')}"
            add_finding(
                findings,
                "high",
                "warm_pool_shortage",
                f"SandboxSet {identity} has {available}/{desired} available replicas",
            )

    set_index = {
        (item.get("namespace"), item.get("name")): item
        for item in sets
    }
    mounted_pv_names = {
        pv_name for sandbox in sandboxes for pv_name in sandbox.get("storagePVs", [])
    }
    storage_pvs = [
        item
        for item in pvs
        if item.get("storageType") and item.get("name") in mounted_pv_names
    ]

    for sandbox in sandboxes:
        if not sandbox.get("storagePVs"):
            continue
        labels = sandbox.get("labels") or {}
        pool_name = labels.get("agents.kruise.io/sandbox-pool")
        pool = set_index.get((sandbox.get("namespace"), pool_name))
        if pool and "csi" not in pool.get("runtimes", []):
            identity = f"{sandbox.get('namespace')}/{sandbox.get('name')}"
            add_finding(
                findings,
                "high",
                "storage_runtime_missing",
                f"Sandbox {identity} declares storage but its SandboxSet lacks the csi runtime",
            )

    for item in storage_pvs:
        name = item.get("name")
        if item.get("phase") in {"Failed", "Released"}:
            add_finding(
                findings,
                "high",
                "storage_pv_unhealthy",
                f"Storage PV {name} is {item.get('phase')}",
            )
        if item.get("volumeHandle") != name:
            add_finding(
                findings,
                "high",
                "storage_volume_handle_mismatch",
                f"Storage PV {name} volumeHandle must equal the PV name",
            )
        if item.get("reclaimPolicy") != "Retain":
            add_finding(
                findings,
                "medium",
                "storage_reclaim_policy",
                f"Storage PV {name} does not use Retain",
            )
        if item.get("storageType") == "oss" and item.get("authType") != "agent-identity":
            add_finding(
                findings,
                "high",
                "oss_agent_identity_missing",
                f"OSS PV {name} does not declare authType=agent-identity",
            )
        if item.get("storageType") == "nas" and "vers=3" not in item.get("mountOptions", []):
            add_finding(
                findings,
                "medium",
                "nas_nfs_version",
                f"NAS PV {name} does not explicitly select NFSv3",
            )

    for item in claims:
        if item.get("phase") in {"Failed", "Error"}:
            identity = f"{item.get('namespace')}/{item.get('name')}"
            add_finding(findings, "high", "claim_failed", f"SandboxClaim {identity} failed")

    for item in updates:
        if item.get("phase") == "Failed" or (item.get("failed") or 0) > 0:
            identity = f"{item.get('namespace')}/{item.get('name')}"
            add_finding(findings, "high", "upgrade_failed", f"SandboxUpdateOps {identity} has failures")

    for pod in report.get("components", {}).get("pods", []):
        if pod.get("phase") != "Running" or any(not c.get("ready") for c in pod.get("containers", [])):
            add_finding(
                findings,
                "high",
                "component_not_ready",
                f"Component Pod sandbox-system/{pod.get('name')} is not fully ready",
            )

    warning_count = report.get("warnings", {}).get("count", 0)
    if warning_count:
        add_finding(findings, "medium", "warning_events", f"Found {warning_count} warning events")

    if not findings:
        add_finding(findings, "info", "no_immediate_findings", "No immediate health findings from bounded inspection")
    return findings


def main() -> int:
    args = parse_args()
    kubectl = Kubectl(args)
    report: dict[str, Any] = {
        "schemaVersion": 1,
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "scope": {"context": args.context, "namespace": args.namespace or "all"},
        "commandsAreReadOnly": True,
        "secretsRead": False,
        "resources": {},
        "errors": [],
    }

    context = kubectl.run(["config", "current-context"])
    report["currentContext"] = context.get("stdout", "").strip() if context["ok"] else None
    if not context["ok"]:
        report["errors"].append({"stage": "current-context", "error": context["error"]})

    available_result = kubectl.run(["api-resources", "-o", "name"])
    available = set(available_result.get("stdout", "").splitlines()) if available_result["ok"] else set()
    if not available_result["ok"]:
        report["errors"].append({"stage": "api-resources", "error": available_result["error"]})

    for resource in [*NAMESPACED_RESOURCES, *CLUSTER_RESOURCES]:
        if available and resource not in available:
            report["resources"][resource] = {"installed": False, "items": []}
            continue
        command = ["get", resource]
        if resource in NAMESPACED_RESOURCES:
            command.extend(["-n", args.namespace] if args.namespace else ["-A"])
        command.extend(["-o", "json"])
        result = kubectl.json(command)
        if result["ok"]:
            items = [summarize_resource(resource, item) for item in result["data"].get("items", [])]
            report["resources"][resource] = {"installed": True, "count": len(items), "items": items}
        else:
            report["resources"][resource] = {
                "installed": resource in available,
                "items": [],
                "error": result["error"],
            }
            report["errors"].append(
                {"stage": f"get {resource}", "error": result["error"]}
            )

    pods_result = kubectl.json(["get", "pods", "-n", "sandbox-system", "-o", "json"])
    report["components"] = {"pods": summarize_pods(pods_result["data"]) if pods_result["ok"] else []}
    if not pods_result["ok"]:
        report["components"]["error"] = pods_result["error"]
        report["errors"].append(
            {"stage": "get sandbox-system pods", "error": pods_result["error"]}
        )

    service_result = kubectl.json(["get", "service", "-n", "sandbox-system", "-o", "json"])
    if service_result["ok"]:
        report["components"]["services"] = [
            {
                **safe_metadata(item),
                "type": (item.get("spec") or {}).get("type"),
                "clusterIP": (item.get("spec") or {}).get("clusterIP"),
                "ports": (item.get("spec") or {}).get("ports") or [],
            }
            for item in service_result["data"].get("items", [])
        ]
    else:
        report["components"]["serviceError"] = service_result["error"]
        report["errors"].append(
            {"stage": "get sandbox-system services", "error": service_result["error"]}
        )

    report["warnings"] = {"count": 0, "items": []}
    if not args.no_events:
        events_result = kubectl.json(
            ["get", "events", "-A", "--field-selector", "type=Warning", "-o", "json"]
        )
        if events_result["ok"]:
            event_items = events_result["data"].get("items", [])
            report["warnings"] = {
                "count": len(event_items),
                "items": [
                    {
                        **safe_metadata(item),
                        "reason": item.get("reason"),
                        "message": item.get("message"),
                        "involvedObject": item.get("involvedObject"),
                        "count": item.get("count"),
                        "lastTimestamp": item.get("lastTimestamp"),
                    }
                    for item in event_items[-100:]
                ],
            }
        else:
            report["warnings"]["error"] = events_result["error"]
            report["errors"].append(
                {"stage": "get warning events", "error": events_result["error"]}
            )

    report["findings"] = evaluate(report)

    rendered = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
        print(f"Wrote redacted inspection report to {args.output}")
    else:
        sys.stdout.write(rendered)

    return 0 if not report["errors"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
