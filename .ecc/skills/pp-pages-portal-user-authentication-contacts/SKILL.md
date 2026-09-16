---
name: pp-pages-portal-user-authentication-contacts
description: "Authentication mapping: Contact records, Entra External ID, Google, and LinkedIn OAuth providers."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-18"]
triggers: ["portal-user-auth-contacts", "entra-external-id-pages", "social-login-providers"]
---

# pp-pages-portal-user-authentication-contacts

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Modern Websites with Microsoft Power Pages - Nick Doelman'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pages-portal-user-authentication-contacts workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pages-portal-user-authentication-contacts processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Modern Websites with Microsoft Power Pages - Nick Doelman*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pages-portal-user-authentication-contacts protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pages-portal-user-authentication-contacts directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pages-portal-user-authentication-contacts execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("portal-user-auth-contacts", "entra-external-id-pages", "social-login-providers") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
