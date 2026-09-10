# Observability and inspection

## Prometheus setup

Agent Sandbox monitoring requires controller `>= v0.5.14` and manager `>= v0.6.1` in the checked documentation.

- Sandbox Controller metrics are exposed through Kubernetes API Server `/metrics` with query parameters `hosting=true` and `job=agent-sandbox-controller`.
- Sandbox Manager metrics are exposed on service port `manager` (HTTP 8080 `/metrics`) in `sandbox-system`.
- Alibaba Cloud Prometheus provides Agent Sandbox integration and three dashboards: Sandbox Instance, Sandbox Controller, and Sandbox Manager.
- Self-managed Prometheus can use scrape configs or ServiceMonitor objects. Ensure ServiceMonitor labels match the Prometheus Operator selector.
- Alibaba Cloud Prometheus treats these as custom metrics and can incur charges.

## Key signals

Instance and pool:

- `sandbox_status_phase`, `sandbox_status_ready`, `sandbox_status_ready_time`
- `sandbox_status_unpaused`, `sandbox_status_unpaused_time`
- `sandbox_status_inplace_updating`, `sandbox_status_inplace_updating_time`
- `sandboxset_replicas`, `sandboxset_available_replicas`, `sandboxset_desired_replicas`

Manager operations:

- `sandbox_claim_creation_responses`, `sandbox_claim_duration_seconds`, `sandbox_claim_retries`
- `sandbox_clone_duration_seconds`, `sandbox_delete_duration_seconds`, `sandbox_pause_duration_seconds`, `sandbox_resume_duration_seconds`, `sandbox_snapshot_duration_seconds`
- `sandbox_routes`, `sandbox_peers`, `sandbox_route_sync_duration_seconds`, `sandbox_route_sync_total`

Control health:

- `controller_runtime_reconcile_errors_total`, `controller_runtime_active_workers`
- `workqueue_depth`, `workqueue_unfinished_work_seconds`, `workqueue_longest_running_processor_seconds`
- `rest_client_requests_total`, `up`, `process_resident_memory_bytes`, `process_open_fds`, `go_goroutines`

Status metrics appear only after the corresponding state has occurred; an empty time series can be normal.

## Suggested alert intent

- Warm-pool shortage: desired exceeds available for a sustained window.
- Lifecycle failure: failed phase/condition or error response increases.
- Reconciliation failure: reconcile errors or workqueue depth grows.
- Gateway health: non-1xx/2xx response codes, `UH`, `UF`, or `UT` flags; route/peer count drops unexpectedly.
- Quota fail-open: manager log states Redis is not configured/unavailable and limited keys are unenforced.
- Capacity risk: vSwitch free IP or enterprise security-group ENI use exceeds planned threshold.
- Identity/storage: token status abnormal, credential provider unavailable, or CSI sidecar reports credential/mount errors.

## Virtual Node metric sharding

Enable at roughly thousand-Pod scale using `kube-system/acs-profile.data.metricsShardSize` (maximum 48). Official guidance: 4 shards for thousands, 24 for 50,000, and 48 for 100,000 online Serverless Pods.

After enabling, scrape `/metrics/cadvisor/<index>`; the original `/metrics/cadvisor` no longer returns cAdvisor data. Alibaba Cloud Prometheus requires probe `>= v2.1.12`. Verify all shard targets are up and use `total_shard_num` and `current_shard_pods_num`.

## Inspection order

1. Confirm kubectl context and API reachability.
2. Check required CRDs and component Pods/images.
3. Summarize Sandbox phase, Ready condition, claimed state, and age.
4. Compare SandboxSet desired/available/updated counts.
5. Check claims, checkpoints, upgrades, restarts, and warning events.
6. Review gateway Service/endpoints and policy CRs.
7. Review Agent Identity, credential provider, storage, and quota backend health without reading secret data.
8. Review metrics availability and alerts.
9. Correlate controller, manager, gateway, traffic-extension, identity, and CSI logs by request ID/time.

Use the bundled script for steps 1-6. It never requests Secret contents.
