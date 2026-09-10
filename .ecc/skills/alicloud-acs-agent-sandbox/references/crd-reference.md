# CRD and state reference

## Core resources

| Resource | API version | Scope/purpose | Common short name |
|---|---|---|---|
| Sandbox | `agents.kruise.io/v1alpha1` | One sandbox and its Pod template/lifecycle | `sbx` |
| SandboxSet | `agents.kruise.io/v1alpha1` | Warm pool and rolling updates | `sbs` |
| SandboxClaim | `agents.kruise.io/v1alpha1` | Allocate replicas from a template | `sbc` |
| Checkpoint | `agents.kruise.io/v1alpha1` | Filesystem snapshot | - |
| SandboxUpdateOps | `agents.kruise.io/v1alpha1` | Upgrade claimed sandboxes | `suo` |
| ContainerRecreateRequest | `apps.kruise.io/v1alpha1` | Restart selected regular containers | - |
| TrafficPolicy | `network.alibabacloud.com/v1alpha1` | Namespace L3/L4 policy | `tp` |
| GlobalTrafficPolicy | `network.alibabacloud.com/v1alpha1` | Cluster L3/L4 baseline | `gtp` |
| SecurityProfile | `agents.kruise.io/v1alpha1` | Namespace L7 egress policy | `sp` |
| GlobalSecurityProfile | `agents.kruise.io/v1alpha1` | Cluster L7 egress baseline | `gsp` |
| AgentIdentity family | `agentidentity.alibabacloud.com/v1alpha1` | Agent identity, credentials, roles, bindings | - |

## Sandbox spec

- `paused`: pause (`true`) or resume (`false`).
- `persistentContents`: currently only `filesystem`.
- `shutdownTime`: RFC3339 absolute deletion time with timezone.
- `pauseTime`: RFC3339 absolute automatic-pause time.
- `runtimes[].name`: `csi`, `agent-runtime`, or `traffic-proxy`.
- `template`: inline Kubernetes `PodTemplateSpec`.
- `volumeClaimTemplates`: PVC templates.

Sidecar injection is effective only for newly created Sandbox instances. `agent-runtime` is required for E2B command and filesystem interfaces. `csi` introduces privileged and `/var/run/csi` hostPath requirements and therefore changes the security boundary.

## Sandbox phases and conditions

Phases: `Pending`, `Running`, `Paused`, `Resuming`, `Upgrading`, `Succeeded`, `Failed`, `Terminating`.

Important conditions: `Ready`, `SandboxPaused`, `SandboxResumed`, `InplaceUpdate`, `Upgrading`, `RuntimeInitialized`. Require condition status `True` for the claimed success state; inspect `reason` and `message` on failure.

## Labels and annotations

| Key | Meaning |
|---|---|
| `agents.kruise.io/sandbox-claimed` | `true` when allocated; `false` in warm pool |
| `agents.kruise.io/sandbox-pool` | Owning SandboxSet |
| `agents.kruise.io/sandbox-template` | Referenced template |
| `agents.kruise.io/claim-name` | Associated SandboxClaim |
| `alibabacloud.com/acs` | Select ACS compute |
| `alibabacloud.com/compute-class` | Use `agent-sandbox` |
| `alibabacloud.com/compute-qos` | Documented quality, commonly `default` |
| `network.alibabacloud.com/vswitch-ids` | Comma-separated sandbox vSwitch IDs |
| `network.alibabacloud.com/security-group-ids` | Comma-separated sandbox security-group IDs |
| `network.alibabacloud.com/network-policy-mode` | `network-policy`, `traffic-policy`, or `enhanced-traffic-policy` |
| `network.alibabacloud.com/enable-network-policy-agent` | Enable network policy agent |
| `security.agents.kruise.io/agent-name` | Bind AgentIdentity |
| `security.agents.kruise.io/enable-jwt-auth` | Enable JWT data-plane auth for one Sandbox |
| `ops.alibabacloud.com/pause-enabled` | Preserve writable layer for supported restart flow |
| `checkpoint.alibabacloud.com/restore-from` | Restore from Checkpoint ID |

## Default kubectl columns

`kubectl get sandbox` exposes `NAME`, `STATUS`, `AGE`, `CLAIMED`, `SHUTDOWN_TIME`, `PAUSE_TIME`, and `MESSAGE`. Do not use phase alone: confirm `Ready`, lifecycle-specific conditions, and the underlying Pod.
