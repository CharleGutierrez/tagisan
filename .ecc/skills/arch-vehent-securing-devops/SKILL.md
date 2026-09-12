---
name: arch-vehent-securing-devops
description: "Cloud security architecture: Defense in depth, automated CI/CD pipeline security gating, secret management with dynamic rotation, and infrastructure-as-code hardening."
triggers: ["securing-devops", "defense-in-depth", "secret-management", "cloud-security", "pipeline-security-gating", "vault-rotation", "iac-security"]
---

# arch-vehent-securing-devops
> Based on **Securing DevOps: Security in the Cloud - Julien Vehent**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Zero secrets (API keys, private keys, database passwords) committed in code, git history, or static configuration files.**
2. **ALWAYS: Inject credentials dynamically at runtime using ephemeral secret managers (HashiCorp Vault, AWS Secrets Manager) with short TTLs and automated rotation.**
3. **NEVER: Run container processes as root (`USER 0`); enforce non-root unprivileged container execution.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Automate security scanning (SAST, DAST, dependency vulnerability scanning) in CI/CD. Enforce ephemeral secret injection and non-root container runtimes.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Hardcoding API keys or database passwords in source code or docker images.**
- **Running containers with full root capabilities and host filesystem access.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-vehent-securing-devops"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
