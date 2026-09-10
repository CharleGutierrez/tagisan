---
name: "sre-triage-access-denied"
description: "Identify why an AWS API call is denied and what policy element blocks it."
---
## When to use

Use when you receive `AccessDenied`, `UnauthorizedOperation`, or `not authorized to perform` errors.

## Preconditions

- You can access the AWS account with permission to inspect IAM and CloudTrail.
- You can identify the failing action and, ideally, the principal and resource.

## Procedure

1. Capture the failing API action, resource, region, and principal (role/user).
2. Look up the corresponding CloudTrail event (or application logs) for context.
3. Identify which policy types apply: identity policy, resource policy, permission boundary, SCP, session policy.
4. Use policy simulation to confirm the explicit deny/implicit deny.
5. Propose the narrowest change that grants access.

## Decision points

- Explicit Deny in any layer: must remove/override deny (cannot be allowed elsewhere).
- SCP denies: change must happen at org/OU level.
- Resource policy missing principal: update resource policy rather than identity.
- Permission boundary blocks: adjust boundary or use a different role.

## Verification

- Re-run the exact API call and confirm success.
- Confirm no broader permissions were granted than necessary.

## Rollback / undo

- Revert the policy change to the prior version.
- Detach any temporary policies used for debugging.

## Escalation

- Security/IAM owners for SCP/boundary changes.
- Service owners for resource policy changes on shared resources.

## Examples

```text
Action: s3:GetObject
Resource: arn:aws:s3:::<bucket>/<key>
Principal: arn:aws:sts::<account>:assumed-role/<role>/<session>
```
