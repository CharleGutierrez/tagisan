---
name: "sre-diagnose-crashloop"
description: "Triage pods restarting repeatedly and identify the most likely root cause."
---
## When to use

Use when a pod is in `CrashLoopBackOff` or restarting and you need to determine why.

## Preconditions

- You have read access to the cluster and target namespace.
- You know the namespace and either a pod name or the owning workload.

## Procedure

1. Identify the crashing container and restart count.
2. Inspect last termination reason/exit code and recent events.
3. Pull logs for the previous container instance.
4. Classify the failure mode (config, dependency, permissions, resource, app bug).
5. Identify the minimal safe remediation and how to verify.

## Decision points

- Exit code `137` / `OOMKilled`: investigate memory limits/usage and leaks.
- Exit code `1` + stack trace: likely application/config issue.
- Readiness/liveness probe failures: check probe config and upstream dependencies.
- Image-related events: see `diagnose-imagepullbackoff`.

## Verification

- Pod transitions to `Running` and stays stable for multiple probe periods.
- Error rate and latency for the workload return to baseline.
- Restart count stops increasing.

## Rollback / undo

- If a remediation involved a rollout, rollback to the previous known-good revision.
- If configuration was changed, revert the config and redeploy.

## Escalation

- Escalate to service owner if the issue is a code regression or requires a patch.
- Escalate to platform team if it appears cluster-wide (many namespaces affected).

## Examples

Example investigation flow (parameterize values):

```bash
kubectl -n <ns> get pod <pod> -o wide
kubectl -n <ns> describe pod <pod>
kubectl -n <ns> logs <pod> --all-containers --previous --tail=200
```
