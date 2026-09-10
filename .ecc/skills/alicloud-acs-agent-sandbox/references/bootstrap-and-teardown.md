# Bootstrap and complete teardown

Use this orchestration when the request starts before the VPC exists or ends after every dedicated resource is released.

## Bootstrap

1. Use `$aliyun-vpc-manage` to inspect zones and create a non-overlapping VPC plus at least two VSwitches in different zones.
2. Use `$alicloud-acs-cluster` to create an ACS cluster with `profile=Acs`, `cluster_spec=ack.pro.small`, the existing VPC/VSwitch IDs, a non-overlapping Service CIDR, enterprise security group, SNAT when public image access is required, and a public API endpoint only when the operator is outside the VPC.
3. Wait for cluster `state=running`, obtain a short-lived KubeConfig, and verify virtual nodes and system Pods.
4. Activate Agent Sandbox and grant `AliyunCSManagedAgentSandboxRole` through the documented one-time console authorization. Use browser automation only in the authenticated Alibaba Cloud console and pause for service agreements, billing choices, or permission grants.
5. Verify `managed-coredns` is installed and `active` before Agent Sandbox components. This is mandatory for Agent Identity storage even when the base E2B path appears healthy. Enable RRSA before installing `ack-agent-identity`; then install or upgrade ALB Ingress Controller, `ack-agent-identity`, `ack-agent-sandbox-controller`, and `ack-sandbox-manager` to versions supported by the current documentation. Discover addon versions and configuration schemas at execution time; do not pin stale component versions.
6. Verify authoritative DNS delegation and China mainland ICP filing before provisioning a public ALB and certificate. Configure the manager domain, TLS, Ingress, and admin API key only after these gates pass. Keep the key out of commands, logs, and evidence; never inspect Manager Pod arguments with `kubectl describe`. For acceptance when no filed domain is available, use the private HTTP protocol through a local port-forward instead of weakening TLS.
7. Create the SandboxSet, wait for all replicas available, allocate one Sandbox, and exercise E2B code, file, and command APIs.
8. Run the bundled inspector and save sanitized evidence.

## Complete teardown

Release in dependency order:

1. Kill allocated E2B Sandbox instances and delete SandboxClaims.
2. Delete Checkpoints, update operations, policies, SandboxSets, standalone Sandboxes, PVCs/PVs, and LoadBalancer Services created for the test. Delete Ingress before its AlbConfig while the ALB controller is still installed; wait for both finalizers and the ALB cloud resource to disappear, then delete the IngressClass.
3. Confirm no test workloads or external endpoints remain.
4. Inventory cluster and addon cloud resources with `$alicloud-acs-cluster`.
5. Disable cluster deletion protection only after exact-ID confirmation, then delete the cluster and explicitly select the intended ALB/SLS/PrivateZone behavior.
6. Poll until the cluster no longer exists and verify auto-created EIP, load balancer, NAT/SNAT, security group, addon resources, RRSA OIDC provider, and ACS Pod ENIs. Cluster disappearance can precede asynchronous ACS ENI garbage collection; do not delete a VSwitch while any interface remains.
7. Use `$aliyun-vpc-manage` to delete the two VSwitches, then the VPC. If deletion reports dependencies, stop and inventory ENIs, load balancers, NAT/EIP, route tables, and security groups; do not force or skip ahead.
8. Remove the local KubeConfig and any temporary credential files, keeping only sanitized acceptance evidence.

Never delete the VPC before the cluster and its network interfaces are fully gone.
