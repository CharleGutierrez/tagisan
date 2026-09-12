---
name: arch-jin-designing-web-apis
description: "Production REST API architecture: Resource-oriented design, idempotency keys, cursor-based pagination, rate limiting (Token Bucket), versioning, and webhook security."
triggers: ["designing-web-apis", "restful-design", "idempotency-key", "cursor-pagination", "rate-limiting", "token-bucket", "webhook-signatures"]
---

# arch-jin-designing-web-apis
> Based on **Designing Web APIs - Brenda Jin, Saurabh Sahni, Amir Shevat**

## 1. Core Architectural Theoretical Foundations & Formal Invariants

1. **ALWAYS: State-altering POST/PATCH requests must require client-provided `Idempotency-Key` headers with cached response replay on duplicate submissions.**
2. **ALWAYS: Paginate large collections using opaque Cursor-based pagination (`limit` & `starting_after`), never SQL `OFFSET` pagination.**
3. **ALWAYS: Verify incoming webhook authenticity using HMAC-SHA256 signatures with timestamp anti-replay validation.**

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)

### Prompt Contract
Design REST APIs with idempotent mutations, cursor pagination, token-bucket rate limiting, and HMAC-signed webhooks. Return standardized RFC 7807 Problem Details.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents

- **Using offset-based pagination (`OFFSET 100000`) causing quadratic database scan degradation.**
- **Non-idempotent billing or checkout endpoints leading to duplicate customer charges.**

## 4. Executable Verification Recipe

```bash
# Verify skill presence and discoverability in Tagisan
tgs ecc skills -q "arch-jin-designing-web-apis"

# Execute automated architectural audit
cargo test --test arch_skills_brutal_tests
```
