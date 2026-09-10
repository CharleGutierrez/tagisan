# Credential Resolution

The CLI binary uses Alibaba Cloud's default credential chain via `credentials.NewCredential(nil)`.

## Resolution Order

| Priority | Source | Description |
|----------|--------|-------------|
| 1 | `ALIBABA_CLOUD_ACCESS_KEY_ID` + `ALIBABA_CLOUD_ACCESS_KEY_SECRET` | Environment variables |
| 2 | `ALIBABA_CLOUD_ACCESS_KEY_ID` + `ALIBABA_CLOUD_ACCESS_KEY_SECRET` + `ALIBABA_CLOUD_SECURITY_TOKEN` | STS token via env vars |
| 3 | `ALIBABA_CLOUD_ROLE_ARN` + `ALIBABA_CLOUD_OIDC_PROVIDER_ARN` + `ALIBABA_CLOUD_OIDC_TOKEN_FILE` | OIDC RAM role (RRSA) |
| 4 | `~/.aliyun/config.json` | Aliyun CLI profile |
| 5 | `~/.alibabacloud/credentials` | INI-style credentials file |
| 6 | ECS metadata `http://100.100.100.200` | ECS RAM role |
| 7 | `ALIBABA_CLOUD_CREDENTIALS_URI` | External credentials service |

## Region

- `REGION_ID` env var (highest priority)
- Auto-detected from ECS metadata on Alibaba Cloud instances

## Endpoint

- `ENDPOINT_TYPE=Vpc` (default): uses `kms-vpc.{region}.aliyuncs.com` for intranet
- `ENDPOINT_TYPE=Public`: uses `kms.{region}.aliyuncs.com` for public internet

## Quick Fix

```bash
export REGION_ID="cn-hangzhou"
export ALIBABA_CLOUD_ACCESS_KEY_ID="your-ak"
export ALIBABA_CLOUD_ACCESS_KEY_SECRET="your-sk"
export ENDPOINT_TYPE="Public"  # if not on Alibaba Cloud VPC
```
