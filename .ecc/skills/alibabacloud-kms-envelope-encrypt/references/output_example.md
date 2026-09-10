# Envelope Encrypt Output Example

When encrypting the string "hello kms" with KMS key `7906979c-8e06-46a2-be2d-68e3ccbc****`,
the output is three newline-separated base64 lines:

```
CiQAK2hMS0xNQUtNUy0wMDE6QUVTMjU2LUdDTTowMTo...
MTIzNDU2Nzg5MDEy
cGF5bG9hZEVuY3J5cHRlZERhdGE=
```

Line 1: KMS CiphertextBlob (encrypted data key) -- base64, variable length
Line 2: Nonce (IV) -- 12 bytes, base64 = 16 chars
Line 3: Ciphertext -- AES-256-GCM encrypted payload, base64, variable length

The actual values differ each time because the data key and nonce are random.
Only the format (3 lines, each valid base64) is predictable.
