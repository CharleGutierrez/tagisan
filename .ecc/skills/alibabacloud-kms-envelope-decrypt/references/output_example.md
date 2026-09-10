# Envelope Decrypt Output Example

Decrypting a valid 3-line envelope ciphertext produces the original plaintext:

Input (`encrypted.enc`):
```
CiQAK2hMS0xNQUtNUy0wMDE6QUVTMjU2LUdDTTowMTo...
MTIzNDU2Nzg5MDEy
cGF5bG9hZEVuY3J5cHRlZERhdGE=
```

Command:
```bash
bash scripts/run.sh decrypt --in-file encrypted.enc
```

Output (stdout):
```
hello kms
```

If `--encryption-context '{"app":"billing"}'` was used during encryption,
the same context must be provided during decryption, otherwise KMS returns:
```
Error: KMS Decrypt: ...
```
