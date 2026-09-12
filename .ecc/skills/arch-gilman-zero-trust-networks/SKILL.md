---
name: arch-gilman-zero-trust-networks
description: "Perimeterless zero trust architecture: Mutual TLS (mTLS), cryptographic machine identities (SPIFFE/SPIRE), microsegmentation, continuous authentication, and dynamic authorization."
triggers: ["zero-trust-networks", "mtls", "spiffe-spire", "microsegmentation", "continuous-authentication", "perimeterless-security"]
---

# arch-gilman-zero-trust-networks
> Based on **Zero Trust Networks - Evan Gilman & Doug Barth**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Enforce Mutual TLS (mTLS) with cryptographically validated machine identities (e.g. SPIFFE IDs) for all service-to-service communication.**
2. **ALWAYS: Authorize every network request dynamically based on caller identity, target resource, and context (Least Privilege access control).**
3. **NEVER: Rely on IP addresses or network CIDR blocks as proof of caller identity.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement Zero Trust data planes using mTLS and SPIFFE/SPIRE certificates with automated rotation (e.g. hourly/daily). Enforce microsegmentation policies.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Relying on IP whitelists for authentication in elastic cloud environments.**
- **Unencrypted plain HTTP communication between internal microservices.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-gilman-zero-trust-networks"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
