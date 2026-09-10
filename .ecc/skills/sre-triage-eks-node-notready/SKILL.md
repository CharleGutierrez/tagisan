---
name: "sre-triage-eks-node-notready"
description: "Diagnose EKS worker nodes in NotReady and determine safe remediation."
---
## When to use

Use when EKS nodes show `NotReady`, `Unknown`, or pods cannot schedule due to node health.

## Preconditions

- You can access Kubernetes node status and AWS instance health.

## Procedure

1. Confirm which nodes are affected and whether it is a single nodegroup/az.
2. Inspect node conditions and kubelet-related events.
3. Identify whether the node is reachable and if CNI is failing.
4. Check EC2 instance status checks and recent scaling events.
5. Decide between waiting, draining, or replacing the node.

## Decision points

- Single node failure: drain/replace after confirming workloads can move.
- Many nodes in same nodegroup/az: suspect networking, IAM, AMI, or a rollout.
- CNI failure symptoms: verify VPC CNI health and ENI/IP exhaustion.

## Verification

- Nodes return to `Ready` or are replaced and the new nodes are `Ready`.
- Pending pods schedule and workload health stabilizes.

## Rollback / undo

- Revert recent nodegroup/AMI/config changes if correlated.
- If replacement worsens impact, pause nodegroup rollouts.

## Escalation

- Platform team for cluster-wide networking/CNI issues.
- AWS support if underlying EC2 issues persist.

## Examples

```bash
kubectl get nodes
kubectl describe node <node>
```
