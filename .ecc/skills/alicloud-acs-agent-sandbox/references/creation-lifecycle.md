# Creation and lifecycle operations

## Contents

- Prerequisites
- Access protocols
- Allocation and replacement
- Pause and resume
- Checkpoint and clone
- Upgrade
- Container restart
- Delete

## Prerequisites

For a new ACS cluster deployment, verify current requirements on the official creation page. The documentation checked on 2026-08-11 requires:

- `acs-virtual-node >= v2.17.0` for creation and Checkpoint; container restart requires `>= v2.18.0`.
- A supported Kube Scheduler build for the cluster minor version.
- `ack-agent-sandbox-controller >= v0.5.14-release.1` and `ack-sandbox-manager >= v0.6.0` for the current E2B integration path.
- More recent features require later versions: pause retention through E2B needs manager `>= 0.6.7`; Agent Identity OSS needs controller `>= v0.5.22-release.1` and manager `>= v0.6.8`.

Never generalize one feature's minimum version to every feature. Check the relevant official page before an upgrade.

Install an ACS-supported Ingress Controller and configure HTTPS. The manager creates an Ingress named `sandbox-manager` in `sandbox-system`. With ALB, configure an HTTPS 443 listener on both `AlbConfig` and Ingress.

## Access protocols

- Native E2B protocol: API at `api.<domain>` and data plane at `<port>-<sandbox-id>.<domain>`. Requires wildcard DNS and TLS and is preferred for production.
- Private protocol: API at `<domain>/kruise/api` and data plane at `<domain>/kruise/<sandbox-id>/<port>`. Requires the `kruise-agents` extension but only one domain; use for tests and simpler integration.

The manager `domain` and client `E2B_DOMAIN` must match exactly and must not include `*`.

## Allocation and replacement

Use `SandboxSet` for a warm pool. The manager discovers it as an E2B template. Verify with:

```bash
kubectl get sandboxset <name> -n <namespace>
kubectl get sandbox -n <namespace> -l agents.kruise.io/sandbox-pool=<name>
```

Allocate through E2B `Sandbox.create(template=...)` or:

```yaml
apiVersion: agents.kruise.io/v1alpha1
kind: SandboxClaim
metadata:
  name: user-a
  namespace: default
spec:
  templateName: code-interpreter
  replicas: 1
  claimTimeout: 5m
  ttlAfterCompleted: 15m
```

Image replacement through E2B uses metadata key `e2b.agents.kruise.io/image`. In-place CPU resizing requires annotation `scaling.alibabacloud.com/enable-inplace-resource-resize: "true"` on the warm-pool template, plus metadata keys `e2b.agents.kruise.io/cpu-request` and `e2b.agents.kruise.io/cpu-limit`. Only CPU is currently honored for this allocation-time resize.

## Pause and resume

Pause only with no active requests or long-lived connections.

E2B automatic pause:

```python
sandbox = Sandbox.create(
    template="code-interpreter",
    timeout=600,
    lifecycle={"on_timeout": "pause"},
    metadata={"e2b.agents.kruise.io/reserve-paused-sandbox-duration": "24h"},
)
```

Manual E2B pause can pass header `x-e2b-kruise-reserve-paused-sandbox-duration`. Durations use Go syntax such as `30m`, `24h`, or `168h`; `7d` is invalid. The default retention is `forever`. Metadata `e2b.agents.kruise.io/never-timeout=true` disables automatic deletion.

Kubernetes operations:

```bash
kubectl patch sandbox <name> -n <namespace> --type=merge -p '{"spec":{"paused":true}}'
kubectl patch sandbox <name> -n <namespace> --type=merge -p '{"spec":{"paused":false}}'
```

Observe `status.phase` and the `SandboxPaused` or `SandboxResumed` condition. CPU and memory are not charged while paused, but all temporary storage is billable. Set `spec.shutdownTime` for deterministic cleanup.

## Checkpoint and clone

Restrictions:

- The Sandbox Pod must be `Running` and `Ready`.
- Only one active Checkpoint per Pod.
- A running Checkpoint cannot be interrupted by deleting its CR.
- Only `filesystem` is supported in `persistentContents`.

E2B snapshot headers include:

- `x-e2b-kruise-snapshot-keep-running` (default `true`)
- `x-e2b-kruise-snapshot-ttl` (unset means persistent until deletion)
- `x-e2b-kruise-snapshot-persistent-contents` (`filesystem` only)
- `x-e2b-kruise-snapshot-wait-success-seconds` (default `60`)

CR form:

```yaml
apiVersion: agents.kruise.io/v1alpha1
kind: Checkpoint
metadata:
  name: checkpoint-code-demo
  namespace: default
spec:
  podName: code-demo
  keepRunning: true
  ttlAfterFinished: 30h
  persistentContents: [filesystem]
```

Read `.status.checkpointId`, then create a Sandbox whose Pod template has annotation `checkpoint.alibabacloud.com/restore-from: <checkpoint-id>`. Keep the cloned Pod spec consistent with the original.

## Upgrade

Warm pool: edit `SandboxSet.spec.template`; control rolling impact with `updateStrategy.maxUnavailable` (default 20%). Completion requires `status.updatedAvailableReplicas == spec.replicas`.

Claimed Sandbox: create one `SandboxUpdateOps` at a time per namespace. It only supports Sandbox phase `Running` or `Upgrading`, rebuilds the Pod, interrupts service, and loses rootfs, memory, and IP. Use `preUpgrade` and `postUpgrade` lifecycle hooks plus OSS/NAS storage for required data.

Diagnose `status.conditions[type=Upgrading].reason`: `PreUpgrade`, `UpgradePod`, `PostUpgrade`, or their `*Failed` forms. A failed operation is one-shot: correct the stage, remove already completed hooks to avoid repetition, delete the old operation, and recreate it.

## Container restart

Use `ContainerRecreateRequest` only when the Pod is `Running`. Init containers are unsupported. The CR must share the Pod namespace.

```yaml
apiVersion: apps.kruise.io/v1alpha1
kind: ContainerRecreateRequest
metadata:
  name: restart-main
  namespace: default
spec:
  podName: <pod-name>
  containers:
    - name: <container-name>
```

Success requires `status.phase=Completed` and each `containerRecreateStates[].phase=Succeeded`. Agent Sandbox rootfs is retained only when annotation `ops.alibabacloud.com/pause-enabled: "true"` is enabled; `emptyDir` and persistent volumes are retained.

## Delete

Use E2B `Sandbox.connect(id).kill()` or `kubectl delete sandbox`. List associated claims, checkpoints, update operations, PV/PVCs, DNS entries, and external resources before deletion. For fleet `PropagationPolicy`, confirm `preserveResourcesOnDeletion`; `false` propagates deletion to member clusters.
