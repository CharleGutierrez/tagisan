# ACK, OpenClaw, and ACK One patterns

## ACK with ACS compute

Install ACK Virtual Node to enable ACS compute, then apply the same Sandbox CRDs and ACS labels. Component and scheduler minimum versions differ from native ACS clusters; recheck the official OpenClaw page for the selected ACK minor version.

For untrusted Agent workloads:

- Use dedicated vSwitches and an enterprise security group, not control-plane vSwitches.
- Combine security-group baselines with GlobalTrafficPolicy and application TrafficPolicy.
- Disable service account token mount and service links, and clear injected Kubernetes service environment variables where the workload does not need them.
- Use a hardened image and a process supervisor appropriate to the application.

The official OpenClaw example builds a persistent-filesystem warm pool, claims an instance through E2B or SandboxClaim, initializes configuration through the runtime API, exposes port 18789, and then uses checkpoint, pause/resume, OSS, and upgrade features. Its permissive UI/auth sample is explicitly not production safe; replace wildcard origins and insecure/device-auth bypasses.

## ACK One fleet

Prerequisites include at least two member clusters, member connectivity, Agent Sandbox components in all members, AMC CLI, and controller `>= v0.5.13-release.1` with feature gate `SandboxMultiClusterNaming=true` to prevent duplicate IDs.

- Distribute a SandboxSet with `PropagationPolicy` using `replicaSchedulingType: Duplicated`; every member receives the complete pool.
- Apply per-cluster replicas/resources through `OverridePolicy` JSON patches.
- Schedule a SandboxClaim with `replicaSchedulingType: Divided` and `customSchedulingType: Gang` so all requested replicas land in one member.
- Use ordered `clusterAffinities` for primary/secondary priority.
- Configure only one PropagationPolicy per SandboxClaim.
- Confirm `preserveResourcesOnDeletion`: `false` deletes member resources when the fleet resource is deleted.
- Automatic failover taints a member `NoSelect` after about five minutes without OCM agent heartbeat. Manual maintenance taints affect already running sandboxes; assess continuity first.

Inspect aggregated and per-member state:

```bash
kubectl get sandboxset,sandboxclaim
kubectl amc get sandboxset -M
kubectl amc get sandboxclaim -M
kubectl amc get sandbox -m <cluster-id> -l agents.kruise.io/claim-name=<claim>
```
