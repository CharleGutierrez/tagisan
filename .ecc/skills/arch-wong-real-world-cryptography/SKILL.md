---
name: arch-wong-real-world-cryptography
description: "Modern applied cryptography: Authenticated Encryption with Associated Data (AEAD: AES-GCM, ChaCha20-Poly1305), password hashing (Argon2id), TLS 1.3, and CSPRNG."
triggers: ["david-wong", "real-world-cryptography", "aead", "aes-gcm", "chacha20-poly1305", "argon2id", "csprng", "tls-1-3"]
---

# arch-wong-real-world-cryptography
> Based on **Real-World Cryptography - David Wong**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Hash user passwords using Argon2id with memory-hard parameters (e.g. 64MB memory, 3 iterations) and a unique per-user salt.**
2. **ALWAYS: Symmetric data encryption must utilize Authenticated Encryption with Associated Data (AEAD, e.g. AES-256-GCM or ChaCha20-Poly1305) with unique nonces.**
3. **NEVER: Implement custom cryptographic algorithms or use broken ciphers (MD5, SHA-1, DES, 3DES, ECB mode).**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Use authenticated encryption (AEAD) for sensitive data at rest and in transit. Hash passwords with Argon2id. Generate random tokens using Cryptographically Secure PRNGs.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Hashing passwords with fast algorithms like MD5, SHA-1, or plain SHA-256.**
- **Using AES in ECB mode without an initialization vector or authentication tag.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-wong-real-world-cryptography"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
