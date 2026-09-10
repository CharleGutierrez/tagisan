---
name: alibabacloud-kms-envelope-encrypt
description: "Envelope encrypt data using KMS (GenerateDataKey + AES-256-GCM). Data stays local\
  \ -- KMS only manages the key, not the data. Trigger when the user wants envelope\
  \ encryption, \u4FE1\u5C01\u52A0\u5BC6, or KMS\u4FE1\u5C01\u52A0\u5BC6. Also for:\
  \ \"protect before git commit,\" \"secure credentials,\" \"encrypt config file,\"\
  \ \"safeguard data before transfer.\" NOT KMS Encrypt API (sends data to KMS, 4KB\
  \ limit). Not for decryption, AWS KMS, gpg, or key management."
---
# Envelope Encrypt

Encrypt data with KMS envelope encryption. On first use, the wrapper script
at `scripts/run.sh` downloads a pre-built binary and caches it at
`~/.cache/alibabacloud-kms-skills/`. Subsequent runs use the cached binary.

## Run

```bash
bash scripts/run.sh encrypt [flags]
```

The script `scripts/run.sh` is in the same directory as this SKILL.md.
If `run.sh` is missing, copy it from the skill's `scripts/` directory.

## Flags

| Flag | Required | Description |
|------|----------|-------------|
| `--key-id` | Yes | KMS CMK ID or alias |
| `--data` | One* | Plaintext string |
| `--in-file` | One* | File path or `-` for stdin |
| `--out-file` | No | Output file, stdout if omitted |
| `--encryption-context` | No | JSON key-value pairs (audit trail, decryption enforces match) |
| `--key-spec` | No | `AES_256` (default) or `AES_128` |
| `--number-of-bytes` | No | Key length 1-1024, overrides `--key-spec` |

*Exactly one of `--data` or `--in-file` is required.

## Output

Three newline-separated base64 lines:

1. Encrypted data key (KMS CiphertextBlob)
2. Nonce (12 bytes)
3. Ciphertext (AES-256-GCM)

Only the format is predictable; values change each run because data keys and
nonces are random. To verify: `test $(wc -l < out.enc) -eq 3`.

## Examples

Encrypt a secret string, output to stdout:
```bash
bash scripts/run.sh encrypt --key-id alias/my-key --data "api-key-abc123"
```

Encrypt a file with audit tagging:
```bash
bash scripts/run.sh encrypt --key-id alias/my-key \
    --encryption-context '{"app":"billing","env":"prod"}' \
    --in-file secrets.yaml --out-file secrets.yaml.enc
```

Encrypt stdin from a pipeline:
```bash
echo "password" | bash scripts/run.sh encrypt --key-id alias/my-key --in-file -
```

## Troubleshooting

**Binary download fails** (`curl` or `wget` error):
The script downloads from GitHub Releases. If blocked by firewall:
- Set `HTTPS_PROXY` if behind a proxy
- Manually download the binary from https://github.com/oobujieshi/alibabacloud-kms-skills-cli/releases
  and place it at `~/.cache/alibabacloud-kms-skills/envelope-encrypt-{platform}`

**Credential or region errors**: Read `references/env_vars.md`.

**"either --data or --in-file is required"**: You didn't specify what to encrypt.
Add `--data "value"` or `--in-file path`.

**"mutually exclusive"**: You passed both `--data` and `--in-file`. Choose one.
