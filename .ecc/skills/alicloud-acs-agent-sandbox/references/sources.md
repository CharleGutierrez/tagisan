# Sources

All sources were checked on 2026-08-11. The Agent Sandbox subtree was expanded from the Help Center sidebar and contains 22 product documents at the time of review.

## Agent Sandbox documentation tree

- [Agent Sandbox overview](https://help.aliyun.com/zh/cs/user-guide/agent-sandbox/) — capabilities, public-preview status, billing, and navigation root.
- [Create Agent Sandbox](https://help.aliyun.com/zh/cs/user-guide/create-an-agent-sandbox) — components, warm pool, E2B/claim allocation, image replacement, CPU resize, TLS, and deletion.
- [Automatic sidecar injection](https://help.aliyun.com/zh/cs/user-guide/configure-automatic-sidecar-injection-for-agent-sandbox) — `csi` and `agent-runtime` injection.
- [Connect with E2B SDK](https://help.aliyun.com/zh/cs/user-guide/connect-to-agent-sandbox-using-the-e2b-sdk) — native/private protocols, supported client versions, and access paths.
- [Sandbox CRD fields](https://help.aliyun.com/zh/cs/user-guide/sandbox-crd-field-descriptions) — Sandbox spec, status, phases, conditions, labels, and kubectl columns.
- [Pause and resume](https://help.aliyun.com/zh/cs/user-guide/hibernate-and-wake-up-the-agent-sandbox) — E2B and CR flows, retention, billing, and limitations.
- [Checkpoint and clone](https://help.aliyun.com/zh/cs/user-guide/clone-agent-sandbox-using-checkpoint) — filesystem Checkpoint, clone, headers, and CR examples.
- [Upgrade warm pools and claimed sandboxes](https://help.aliyun.com/zh/cs/user-guide/upgrade-pre-warmed-pools-and-claimed-sandboxes) — SandboxSet rolling update and SandboxUpdateOps.
- [Restart containers](https://help.aliyun.com/zh/cs/user-guide/restart-agent-sandbox-containers) — ContainerRecreateRequest prerequisites and state retention.
- [Manage API keys and teams](https://help.aliyun.com/zh/cs/user-guide/manage-api-keys-and-teams) — tenant model and HTTP interfaces.
- [API key quota](https://help.aliyun.com/zh/cs/user-guide/set-resource-quota-for-agent-sandbox-api-key-with-sandbox-manager-quota-function) — Redis/Tair-backed count, CPU, and memory enforcement.
- [Store API keys in RDS MySQL](https://help.aliyun.com/zh/cs/user-guide/store-api-keys-with-rds-mysql) — backend selection, schema, pepper, migration, and validation.
- [Mount OSS with Agent Identity](https://help.aliyun.com/zh/cs/mount-oss-storage-for-an-agent-sandbox) — RRSA, managed CoreDNS, identity CRs, PV, dynamic mount, and troubleshooting.
- [Manage ingress with SandboxGateway](https://help.aliyun.com/zh/cs/user-guide/use-sandboxgateway-to-forward-data-plane-traffic) — control/data separation, routing, authentication, metrics, capacity, and rollback.
- [TrafficPolicy](https://help.aliyun.com/zh/cs/user-guide/use-trafficpolicy-to-manage-agent-network-access-1) — L3/L4 rules, priorities, versions, and CRD schema.
- [Enhanced egress traffic management](https://help.aliyun.com/zh/cs/user-guide/manage-agent-sandbox-egress-traffic) — SecurityProfile, L7 matching/actions, TLS, audit, and MCP ACL.
- [Egress credential injection](https://help.aliyun.com/zh/cs/user-guide/inject-credentials-for-agent-sandbox-egress) — API key and Alibaba Cloud STS transformation.
- [Network planning and scaling](https://help.aliyun.com/zh/cs/user-guide/network-planning-and-scaling) — trust zones, vSwitch, SNAT, security groups, annotations, and validation.
- [Prometheus monitoring](https://help.aliyun.com/zh/cs/user-guide/enable-prometheus-monitoring-for-acs-agent-sandbox-1) — collection, dashboards, billing, and metric catalog.
- [Virtual Node metric sharding](https://help.aliyun.com/zh/cs/user-guide/enable-paging-collection-of-virtual-node-monitoring-metrics) — shard sizing and scrape migration.
- [Deploy OpenClaw on ACK with ACS Agent Sandbox](https://help.aliyun.com/zh/cs/user-guide/deploy-openclaw-on-ack-clusters-using-acs-agent-sandbox) — ACK compute, isolation, warm pool, allocation, access, and lifecycle pattern.
- [Manage Agent Sandbox with ACK One fleet](https://help.aliyun.com/zh/cs/user-guide/through-the-ack-one-multi-cluster-fleet-management-agent-sandbox) — propagation, overrides, scheduling, capacity, and failover.

## Related primary sources

- [ACS supported regions](https://help.aliyun.com/zh/cs/product-overview/open-service-area) — confirms Hangzhou as `cn-hangzhou` and other currently supported regions.
- [Create an ACS cluster](https://help.aliyun.com/zh/cs/user-guide/create-an-acs-cluster) — ACS OpenAPI profile, networking, addons, API access, and deletion protection.
- [ACS cluster network planning](https://help.aliyun.com/zh/cs/user-guide/acs-cluster-network-planning) — VPC, VSwitch, Pod IP, and Service CIDR design.
- [ACS NAS storage volumes](https://help.aliyun.com/zh/cs/user-guide/use-nas-storage-volumes) — supported modes, NFSv3 and same-VPC constraints, StorageClass parameters, and persistence validation.
- [ACS static NAS volumes](https://help.aliyun.com/zh/cs/user-guide/mount-statically-provisioned-nas-volumes) — NAS CSI PV fields, mount options, and shared-storage validation.
- [Delete an ACS cluster](https://help.aliyun.com/zh/cs/user-guide/deleting-a-cluster) — workload cleanup and associated-resource deletion sequence.
- [ACS billing rules](https://help.aliyun.com/zh/cs/product-overview/product-billing-rules) — general ACS billing context; Agent Sandbox rates remain on its overview page.
- [OpenKruise Agents](https://github.com/openkruise/agents) — upstream CRD and E2B-compatible implementation.
- [E2B repository](https://github.com/e2b-dev/E2B) — upstream client implementation; ACS-specific supported versions and behavior are governed by Alibaba Cloud documentation.
- [ACK RAM tool](https://github.com/AliyunContainerService/ack-ram-tool) — related Alibaba Cloud container-service RAM tooling inspected during source verification; no skill workflow depends on it.
