---
name: copilot-jwe-token-cryptography
description: "Decrypting RFC 7516 JWE payloads with RSA-OAEP-256 and AES-256-GCM in Graph subscriptions."
version: 1.0.0
tier: "Enterprise / Microsoft 365 Copilot & Vibe Code Development"
source: "JSON Web Encryption (JWE) & Token Decryption in Copilot - RFC 7516 Standard"
tags:
  - copilot
  - m365
  - vibe-coding
  - enterprise
compatibility: ">=0.2.0"
triggers: ["jwe-cryptography", "rfc7516-decryption", "rsa-oaep-256", "aes-256-gcm-tokens"]
---

# copilot-jwe-token-cryptography
> Based on **JSON Web Encryption (JWE) & Token Decryption in Copilot - RFC 7516 Standard** (Cluster 11)

## 1. Core Mathematical & Architectural Foundations / Formal Invariants

1. **Cryptographic Pipeline: Decrypt content encryption key using private RSA key -> Decrypt ciphertext using AES-GCM.**
2. **Authentication Tag: Validate 128-bit authentication tag before parsing decrypted JSON.**
3. **MANDATORY hardware security module (HSM) or Azure Key Vault key storage.**
4. **ALWAYS enforce schema compliance, deterministic typing, and cryptographic audit receipts in copilot-jwe-token-cryptography.**
5. **NEVER bypass Entra ID tenant boundaries, ambient credentials, or unvalidated token claims in copilot-jwe-token-cryptography.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement RFC 7516 JWE decryption routines to securely process encrypted Graph notification payloads.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Bypassing formal verification rules in copilot-jwe-token-cryptography.**
- **Unmonitored runtime execution without telemetry in copilot-jwe-token-cryptography.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "jwe-cryptography"

# Execute automated M365 Copilot engineering audit
cargo test --test ms365_copilot_skills_brutal_tests
```
