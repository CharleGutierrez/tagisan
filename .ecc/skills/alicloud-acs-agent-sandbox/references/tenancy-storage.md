# Tenancy, quotas, and key storage

## Contents

- Team and API key model
- Quota
- MySQL key storage

## Team and API key model

`ack-sandbox-manager >= v0.6.0` maps each non-admin Team name directly to an existing Kubernetes namespace. API keys authenticate every request through `X-API-KEY`.

- A tenant can list its Team and keys, create a key for its own Team, and delete its own keys.
- Admin can list all Teams, create keys for another existing Team, and delete any non-admin key.
- The built-in admin key cannot be deleted.
- A tenant can access only sandboxes created with its own API key; admin can access all.
- API key plaintext is returned only by `POST /api-keys`. Persist it immediately in an approved secret store and never echo it.

Endpoints are direct HTTP interfaces, not E2B SDK or Kubernetes CRDs:

- `GET /teams`
- `GET /api-keys`
- `POST /api-keys` with `name`, optional admin-only `teamName`, and optional `quota`
- `DELETE /api-keys/{uuid}`

Native protocol base is `https://api.<domain>`; private protocol base is `https://<domain>/kruise/api`.

## Quota

Quota is immutable at key creation in the documented version. To change it, issue a replacement key and revoke the old one after cutover.

```json
{
  "quota": {
    "limits": [
      {"dimension": "sandbox.count", "scope": "all", "limit": 10},
      {"dimension": "limits.cpu", "scope": "all", "limit": 8000},
      {"dimension": "limits.memory", "scope": "all", "limit": 16384}
    ]
  }
}
```

CPU uses millicores and memory uses MiB. `scope` currently accepts only `all`. Count includes all live states, including paused; CPU and memory account for running resources.

Quota requires Redis/Tair for atomic cross-replica admission. Configure the control-plane vSwitch CIDR in the Redis allowlist, never the sandbox vSwitch CIDR. If Redis is unavailable, enforcement is fail-open. The default circuit breaker opens after three failures and probes again after 30 seconds; alert on manager logs containing `limited keys are accepted but unenforced`.

## MySQL key storage

Manager `>= v0.6.7` supports `keyStorage.mode=secret` (default) and `mysql`.

- Secret stores recoverable key material in `sandbox-system/e2b-key-store`, is limited by the Kubernetes 1 MiB Secret limit, and is suitable for small deployments.
- MySQL stores `HMAC-SHA256(pepper, rawKey)`, not plaintext, and is recommended above 1,000 keys. Plan migration at 501-1,000 keys.
- Configure `keyStorage.mysql.dsn` and a separate 32-byte `hashPepper`. Rotating the pepper invalidates all existing tenant keys.
- Use MySQL 5.7 or later; MySQL 8.0 is recommended. Permit the control-plane vSwitch CIDRs and use the VPC endpoint.
- Initialize official `teams` and `team_api_keys` schema before switching.

During Secret-to-MySQL migration, pause key creation/deletion, dry-run the official migration script, provide the pepper through hidden input/environment, import the generated SQL, validate create/list/delete, then securely remove the SQL and environment value.
