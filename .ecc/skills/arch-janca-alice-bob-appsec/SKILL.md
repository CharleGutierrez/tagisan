---
name: arch-janca-alice-bob-appsec
description: "Application security fundamentals: OWASP Top 10 mitigations, secure SDLC, input validation allowlists, parameterized queries, and defensive coding practices."
triggers: ["alice-bob-appsec", "tanya-janca", "owasp-top-10", "input-validation", "parameterized-queries", "sql-injection-mitigation", "bola-defense"]
---

# arch-janca-alice-bob-appsec
> Based on **Alice and Bob Learn Application Security - Tanya Janca**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: Validate and sanitize all external inputs using strict allowlists (never blocklists); reject any payload failing validation schemas.**
2. **ALWAYS: Prevent SQL Injection by mandating parameterized queries or prepared statements; string concatenation in SQL queries is strictly prohibited.**
3. **ALWAYS: Enforce object-level authorization checks (BOLA/IDOR prevention) verifying the authenticated user owns the requested resource ID.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Implement defensive coding against OWASP Top 10. Use parameterized queries everywhere. Enforce strict JSON schema input validation and object-level authorization.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Concatenating user input directly into SQL, shell, or LDAP queries.**
- **Trusting client-supplied IDs in URLs without checking user ownership permissions (IDOR).**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-janca-alice-bob-appsec"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
