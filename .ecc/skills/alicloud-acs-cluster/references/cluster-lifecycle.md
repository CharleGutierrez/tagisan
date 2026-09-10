# ACS cluster lifecycle

## Contents

- Create preflight
- Create request
- Readiness and access
- Associated resources
- Teardown

## Create preflight

Complete the account-level CSI service-role authorization before creating a dedicated VPC. New clusters can require `AliyunCSManagedCsiPluginRole` and `AliyunCSManagedCsiProvisionerRole` even when the create request lists only an ingress addon, because CSI components are part of the managed cluster baseline. Each role must trust `cs.aliyuncs.com` and carry its matching Alibaba Cloud system policy. Use the official authorization flow; do not synthesize a broader custom policy.

The bundled `create --dry-run` command renders the local OpenAPI request only. It cannot prove that RAM roles, quotas, capacity, or addon versions are available. Treat those as separate preflight gates.

For ACS, set `cluster_type=ManagedKubernetes`, `profile=Acs`, and `cluster_spec=ack.pro.small`. Use `cn-hangzhou` for Hangzhou. Omit `kubernetes_version` to let the service choose the latest supported version; if pinning a version, query current version metadata first.

Plan non-overlapping ranges. A bounded validation layout is:

- VPC: `10.42.0.0/16`
- VSwitch A: `10.42.0.0/20`
- VSwitch B: `10.42.16.0/20`
- Service CIDR: `172.21.0.0/20`

Check these ranges against existing VPCs, VPN/CEN routes, local networks, and other clusters before creating anything. VSwitches supply Pod IPs, and Service CIDR cannot be changed after cluster creation.

Use `snat_entry=true` when Pods must pull public images or reach public services. Use `endpoint_public_access=true` only when Codex runs outside the VPC and must access the API Server. Both options can create billable network resources.

## Create request

Use existing VPC and VSwitch IDs so ownership is explicit. Request an enterprise security group for Agent Sandbox scale, explicitly request `managed-coredns`, and install `alb-ingress-controller` if the Sandbox manager will be reached through ALB. Do not assume every core addon is installed when the create request supplies an addon list.

Creation normally takes minutes. Treat the returned task ID as submission evidence, then poll `DescribeClusterDetail` until `state=running`. Stop on `failed`, and preserve the task/request IDs for diagnosis.

## Readiness and access

After the cluster reports running:

```bash
kubectl --kubeconfig <protected-path> cluster-info
kubectl --kubeconfig <protected-path> get nodes -o wide
kubectl --kubeconfig <protected-path> get pods -A
```

Require API reachability, at least one Ready virtual node per selected zone, `managed-coredns` addon state `active`, the `kube-dns` Service, DNS resolution from a newly created Pod, and no persistent system Pod failures. Use a short-lived KubeConfig and delete the file after the operation.

## Associated resources

Run `DescribeClusterResources` with addon resources enabled before teardown. Record IDs, types, creator, state, and delete behavior. Typical resources include API Server load balancers, EIP, NAT/SNAT, security groups, ALB, SLS, and PrivateZone resources. The exact list is service-generated and must be discovered rather than assumed.

## Teardown

1. Delete Sandbox claims/instances, Checkpoints, SandboxSets, policies, Ingresses, PVCs/PVs, and remaining workloads.
2. Confirm all namespaced workload controllers and LoadBalancer Services are gone.
3. Inventory cluster and addon cloud resources.
4. Disable deletion protection with exact cluster-ID confirmation.
5. Delete the cluster with explicit associated-resource behavior.
6. Poll until `DescribeClusterDetail` returns not found.
7. Verify every auto-created resource is deleted or deliberately retained.
8. Continue polling VPC ENIs after the cluster object disappears; ACS Pod interfaces can be garbage-collected asynchronously.
9. Delete separately created VSwitches, then the VPC, only after all dependent ENIs, load balancers, NAT/EIP, security groups, and routes are gone.

Never delete the VPC first. Never interpret an accepted delete task as final cleanup.
