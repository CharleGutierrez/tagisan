# OSS and NAS mounts

Use this workflow to attach an existing OSS bucket or NAS file system to a Sandbox allocated through the E2B-compatible API. The Agent Sandbox CSI runtime consumes a pre-created PV by name; it does not consume a PVC from the E2B metadata.

## Choose the backend

| Requirement | Backend | Authentication and network boundary |
|---|---|---|
| Object data, models, artifacts, or bucket prefixes | OSS | Use Agent Identity and short-lived STS credentials. Use the regional HTTPS internal endpoint. |
| Shared POSIX files, high-I/O shared data, or persistence across Sandboxes | NAS | Use an NFS mount target in the same VPC as the Sandbox Pod. ACS supports NFSv3, not SMB. |

Both paths require the `csi` runtime. CSI injection adds a privileged container and mounts host path `/var/run/csi`; enable it only on a dedicated storage-enabled SandboxSet after accepting the isolation impact. Runtime changes affect newly created Sandboxes only.

## Common SandboxSet preparation

Create or update a dedicated SandboxSet with both runtimes. Do not assume an existing Sandbox is reinjected after this change.

```yaml
apiVersion: agents.kruise.io/v1alpha1
kind: SandboxSet
metadata:
  name: code-interpreter-storage
  namespace: default
spec:
  replicas: 1
  runtimes:
    - name: csi
    - name: agent-runtime
  template:
    metadata:
      annotations:
        network.alibabacloud.com/wait-clusterip-ready: "*"
      labels:
        alibabacloud.com/acs: "true"
        alibabacloud.com/compute-class: agent-sandbox
        alibabacloud.com/compute-qos: default
    spec:
      automountServiceAccountToken: false
      dnsPolicy: ClusterFirst
      containers:
        - name: sandbox
          image: <REGION-LOCAL-HARDENED-IMAGE>
          resources:
            requests: {cpu: "1", memory: 1Gi, ephemeral-storage: 30Gi}
            limits: {cpu: "1", memory: 1Gi}
```

Before creating this pool, confirm that the `managed-coredns` addon exists and is `active`, and that the `kube-dns` Service exists. An ACS cluster created with an explicit addon list can omit CoreDNS even though ordinary Sandbox operations still work. The wait annotation does not replace CoreDNS. Existing Manager, Gateway, and Sandbox Pods keep the resolver written at Pod creation, so restart/recreate them after a late CoreDNS installation. Verify a newly created Sandbox has the cluster DNS server and can resolve both `id-provider.ack-agent-identity.svc` and `credential-provider.ack-agent-identity.svc`.

Wait until `status.availableReplicas` equals `spec.replicas`. If the cluster or security group restricts east-west traffic, explicitly allow CoreDNS and the storage-specific endpoints described below.

## Mount OSS with Agent Identity

Do not place AccessKey credentials in a PV, Secret, Sandbox, E2B metadata, or command line. Agent Identity issues per-Sandbox STS credentials through RRSA and enforces the intersection of:

1. the RAM role policy, which is the maximum permission boundary; and
2. the `CredentialProvider` policy, which narrows actual access for the Sandbox.

Current documented prerequisites are:

- `ack-agent-identity >= v0.4.0` with `agentTokenDelegation`;
- `ack-agent-sandbox-controller >= v0.5.22-release.1` with `identityProvider`;
- `ack-sandbox-manager >= v0.6.8` with `identityProvider`;
- RRSA OIDC enabled; and
- RAM role trust restricted to subject `system:serviceaccount:ack-agent-identity:credential-provider`.

Follow the official Agent Identity procedure to create the `AgentIdentity`, `CredentialProvider`, `AgentRole`, and `AgentRoleBinding`. Keep the RAM and provider policies scoped to the required bucket prefix and actions. Then create a PV like this:

```yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: pv-oss-sandbox
spec:
  capacity:
    storage: 10Gi
  accessModes:
    - ReadWriteMany
  persistentVolumeReclaimPolicy: Retain
  csi:
    driver: ossplugin.csi.alibabacloud.com
    volumeHandle: pv-oss-sandbox
    volumeAttributes:
      authType: agent-identity
      bucket: <BUCKET_NAME>
      url: https://oss-cn-hangzhou-internal.aliyuncs.com
      path: /
      otherOpts: "-o sigv4 -o region=cn-hangzhou -o umask=000 -o allow_other"
```

`volumeHandle` must equal the PV name. Do not configure `nodePublishSecretRef`. Treat `volumeAttributes` as immutable; recreate the PV only after all consumers have stopped. Mount a bucket subdirectory rather than a large bucket root to avoid excessive memory use during directory listing. The public code-interpreter image runs as a non-root user: `umask=022` makes a root-owned OSSFS mount readable but not writable to that user. Use a deliberately reviewed mode such as `umask=000` for the isolated acceptance bucket, or set production ownership and permissions to the least privilege supported by the selected image. Do not describe `ReadWriteMany` as proof of POSIX write permission.

Allow Sandbox DNS, `credential-provider.ack-agent-identity.svc:8443`, and the OSS internal endpoint through TrafficPolicy and security groups. Allocate and test:

```bash
python3 skills/compute/acs/alicloud-acs-agent-sandbox/scripts/e2b_smoke_test.py \
  --private-protocol \
  --template code-interpreter-storage \
  --storage-type oss \
  --storage-pv pv-oss-sandbox \
  --storage-mount-path /mnt/oss \
  --storage-sub-path sandbox-acceptance \
  --credential-provider <CREDENTIAL_PROVIDER_NAME> \
  --agent-name <AGENT_IDENTITY_NAME> \
  --output output/alicloud-acs-agent-sandbox/storage/oss-smoke.json
```

For a read-only policy, also pass `--storage-read-only`; the smoke test verifies readability and does not attempt mutation.

## Mount NAS

Confirm the `csi-provisioner` component is installed. The NAS file system must use NFS, its mount target must be `Available` and in the Sandbox VPC, and the access group must permit the Sandbox network. ACS does not support cross-VPC NAS mounts or SMB and supports only NFSv3. Same-VPC cross-zone mounting is supported, but a same-zone mount target is preferable.

Create a static PV for E2B allocation:

```yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: pv-nas-sandbox
spec:
  capacity:
    storage: 20Gi
  accessModes:
    - ReadWriteMany
  persistentVolumeReclaimPolicy: Retain
  csi:
    driver: nasplugin.csi.alibabacloud.com
    volumeHandle: pv-nas-sandbox
    volumeAttributes:
      server: <MOUNT_TARGET>.cn-hangzhou.nas.aliyuncs.com
      path: /agent-sandbox
  mountOptions:
    - nolock,tcp,noresvport
    - vers=3
```

`volumeHandle` must equal the PV name. The declared capacity does not limit actual NAS capacity. For General-purpose NAS the root is `/`; for Extreme NAS the root is `/share`, so a child path must start with `/share`. Avoid `securityContext.fsGroup`, which can delay or fail mounting, and never delete a mount target while it is mounted.

A newly created General-purpose NAS child directory is commonly owned by root. If the Sandbox image runs as non-root, initialize the dedicated directory once from a tightly scoped administrative Pod and set the intended owner/mode before acceptance. Do not solve this by granting broad access to the NAS root. Delete the initialization Pod immediately and record only directory metadata, never file contents.

Allocate and test:

```bash
python3 skills/compute/acs/alicloud-acs-agent-sandbox/scripts/e2b_smoke_test.py \
  --private-protocol \
  --template code-interpreter-storage \
  --storage-type nas \
  --storage-pv pv-nas-sandbox \
  --storage-mount-path /mnt/shared \
  --output output/alicloud-acs-agent-sandbox/storage/nas-smoke.json
```

For dynamically provisioned NAS outside the E2B allocation path, use a StorageClass with provisioner `nasplugin.csi.alibabacloud.com`. `volumeAs: subpath` reuses an existing file system; `volumeAs: filesystem` creates a billable NAS file system. The documented reclaim policy is `Retain`, so deleting the PVC/PV does not by itself release the NAS file system or mount target.

## E2B metadata contract

The smoke script supplies `e2b.agents.kruise.io/csi-volume-config` as a JSON array containing:

- `pvName`: existing PV name;
- `mountPath`: absolute, empty target directory in the Sandbox;
- optional `subPath`;
- `readOnly`; and
- for OSS Agent Identity only, `attributes.credentialProviderName` plus metadata key `security.agents.kruise.io/agent-name`.

The script records only the storage type, PV name, mount path, subpath, read-only state, and Boolean check results. It never reads Kubernetes Secrets or accepts cloud credentials.

## Acceptance and troubleshooting

Pass storage acceptance only when:

1. the PV is `Available` before allocation;
2. the storage-enabled Sandbox reaches `Running` and `Ready=True`;
3. the mount directory exists and is readable;
4. read-write mode can create, read, and remove a unique marker;
5. read-only mode reads successfully without a mutation attempt;
6. Sandbox cleanup succeeds; and
7. no AccessKey, token, kubeconfig, or provider response appears in evidence.

For OSS failures, inspect in order: `managed-coredns` addon state and the new Pod's `/etc/resolv.conf`, provider service resolution/connectivity, Pod annotation `security.agents.kruise.io/token-status`, RRSA trust, CSI sidecar logs, CredentialProvider policy, RAM role policy, and finally mounted POSIX ownership/mode. A failure mentioning lookup through `100.100.2.136` or `100.100.2.138` means the caller was created before cluster DNS became available; install/repair CoreDNS and recreate that Pod. For NAS failures, inspect the CSI sidecar, same-VPC routing, mount-target state, access-group rules, NFSv3 options, directory path, and non-root write permissions.

Concurrent writes to the same OSS object or NAS file require application-level coordination.

## Safe teardown

1. Stop allocating new storage-enabled Sandboxes.
2. Kill claimed Sandboxes and scale the storage SandboxSet to zero.
3. Confirm no Sandbox or CSI sidecar still references the PV.
4. Delete the PV. `Retain` preserves the external data service.
5. For OSS, delete Agent Identity bindings/providers/roles only when no other Sandbox uses them; remove the RAM role last.
6. For NAS, delete the mount target and file system only after checking for other clients and obtaining explicit approval. These are separate billable resources and are not removed by deleting a retained PV.
7. Delete OSS objects or the bucket only with explicit approval and a verified exact target.
