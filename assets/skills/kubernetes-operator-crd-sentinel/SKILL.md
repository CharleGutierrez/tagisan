---
name: kubernetes-operator-crd-sentinel
description: Autonomous Kubernetes Operator and Custom Resource Definition (CRD) controller engine using kube-rs. Enforces strictly idempotent reconciliation loops, atomic status subresource patching, leak-proof finalizer lifecycles, and resilient lease-based leader election failover.
version: 1.0.0
tags:
  - kubernetes
  - kube-rs
  - operator
  - crd-reconciliation
  - finalizers
  - status-subresource
  - leader-election
  - controller-runtime
triggers:
  - kubernetes-operator-crd-sentinel
  - kube-rs-operator
  - crd-controller
  - kubernetes-reconciliation
  - finalizer-sentinel
  - operator-status-patch
compatibility: ">=0.2.0"
---

# Kubernetes Operator & CRD Reconciliation Sentinel

## Purpose & Scope
Cloud-native infrastructure relies on Kubernetes operators and custom controllers to manage complex stateful distributed systems (databases, stream processors, AI model training clusters). Faulty controllers lead to split-brain deployments, resource leaks during namespace deletions, flapping status fields, and thundering herd API server overloads.

This skill provides an autonomous engineering framework for authoring, verifying, and hardening Kubernetes Operators and CRD reconcilers using `kube-rs`. It enforces strict idempotence in reconciliation loops, leak-proof finalizer lifecycles, atomic status subresource patching via Server-Side Apply (SSA), and lease-based leader election failover.

---

## 1. Operational Invariants (Kubernetes Controller Protocol)

### Invariant 1: Strict Idempotence in Controller Reconciliation Loops
- **MANDATORY**: The reconcile loop function `reconcile(obj: Arc<K>, ctx: Arc<Context>)` must be strictly idempotent. Re-running the loop `N` times against the same cluster state must yield identical results without duplicate side-effects.
- **ALWAYS**: Treat the cluster state as observed facts and drive current state toward desired state specified in `spec`.
- **STRICT_REJECT**: Reject any reconciler that assumes sequential linear execution or fails when triggered repeatedly on unchanged objects.

### Invariant 2: Finalizer Lifecycle Safety & Leak-Proof Resource Deletion
- **MANDATORY**: Any custom resource managing external infrastructure (DNS entries, cloud storage buckets, cloud databases) must register a finalizer before provisioning external resources.
- **ALWAYS**: When `metadata.deletion_timestamp` is present, execute external teardown first, verify completion, and only then atomically remove the finalizer.
- **NEVER**: Block finalizer cleanup indefinitely on unrecoverable downstream errors without exponential requeue backoff.
- **STRICT_REJECT**: Reject any controller design that leaves dangling external resources upon CRD instance deletion.

### Invariant 3: Atomic Status Subresource Patching & Optimistic Concurrency Control
- **MANDATORY**: Never update the full resource spec when reporting status. Use Kubernetes status subresource endpoints (`/status`) with Server-Side Apply (`Patch::Apply`) and a dedicated field manager.
- **ALWAYS**: Ensure conditions adhere to Kubernetes standard condition conventions (`type`, `status`, `reason`, `message`, `lastTransitionTime`).
- **STRICT_REJECT**: Reject status updates using optimistic concurrency read-modify-write loops without conflict retry backoff.

### Invariant 4: High-Availability Lease-Based Leader Election Failover
- **MANDATORY**: Highly-available multi-replica operator deployments must enforce leader election using Kubernetes `coordination.k8s.io/v1` Leases.
- **ALWAYS**: Only the active leader replica executes the reconciler loop; standby replicas maintain warm informers.
- **STRICT_REJECT**: Reject multi-replica controller architectures running active-active write loops on the same CRD without partitioned shard keys.

---

## 2. Canonical `kube-rs` Implementation Patterns

### Robust Idempotent Reconciler with Finalizer & Status Subresource (Rust)
```rust
use kube::{
    api::{Api, Patch, PatchParams, ResourceExt},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        finalizer::{finalizer, Event as FinalizerEvent},
    },
    CustomResource,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

pub static FINALIZER_NAME: &str = "sentinel.tagisan.io/cleanup";

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(group = "tagisan.io", version = "v1alpha1", kind = "DatabaseCluster", namespaced)]
#[kube(status = "DatabaseClusterStatus")]
pub struct DatabaseClusterSpec {
    pub replicas: i32,
    pub storage_gb: i32,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct DatabaseClusterStatus {
    pub ready_replicas: i32,
    pub phase: String,
}

struct ContextData {
    client: Client,
}

pub async fn reconcile_cluster(cluster: Arc<DatabaseCluster>, ctx: Arc<ContextData>) -> Result<Action, kube::Error> {
    let client = ctx.client.clone();
    let ns = cluster.namespace().unwrap_or_else(|| "default".to_string());
    let api: Api<DatabaseCluster> = Api::namespaced(client.clone(), &ns);

    finalizer(&api, FINALIZER_NAME, cluster, |event| async {
        match event {
            FinalizerEvent::Apply(c) => reconcile_apply(c, &api).await,
            FinalizerEvent::Cleanup(c) => reconcile_cleanup(c).await,
        }
    }).await.map_err(|e| kube::Error::Service(e.into()))
}

async fn reconcile_apply(cluster: Arc<DatabaseCluster>, api: &Api<DatabaseCluster>) -> Result<Action, kube::Error> {
    // 1. Desired state convergence (idempotent)
    let status_patch = serde_json::json!({
        "status": {
            "ready_replicas": cluster.spec.replicas,
            "phase": "Running"
        }
    });

    api.patch_status(
        &cluster.name_any(),
        &PatchParams::apply("database-cluster-controller"),
        &Patch::Merge(&status_patch)
    ).await?;

    Ok(Action::requeue(Duration::from_secs(300)))
}

async fn reconcile_cleanup(cluster: Arc<DatabaseCluster>) -> Result<Action, kube::Error> {
    // Teardown external cloud resources safely
    println!("Cleaning up external resources for: {}", cluster.name_any());
    Ok(Action::await_change())
}
```
