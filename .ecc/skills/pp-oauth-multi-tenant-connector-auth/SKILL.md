---
name: pp-oauth-multi-tenant-connector-auth
description: "Architecting multi-tenant Entra ID connectors, common endpoints, and tenant admin consent."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-29"]
triggers: ["multi-tenant-connector-auth", "common-endpoint-oauth", "admin-consent-workflow"]
---

# pp-oauth-multi-tenant-connector-auth

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Identity and Authentication in Power Platform - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-oauth-multi-tenant-connector-auth workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-oauth-multi-tenant-connector-auth processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Identity and Authentication in Power Platform - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-oauth-multi-tenant-connector-auth protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-oauth-multi-tenant-connector-auth directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-oauth-multi-tenant-connector-auth execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("multi-tenant-connector-auth", "common-endpoint-oauth", "admin-consent-workflow") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
