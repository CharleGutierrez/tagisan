---
name: pp-oauth-api-key-and-basic-authentication
description: "Configuring API Key header/query authentication and secure Basic auth connection settings."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-29"]
triggers: ["api-key-basic-auth", "api-key-header-security", "connector-credential-storage"]
---

# pp-oauth-api-key-and-basic-authentication

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Identity and Authentication in Power Platform - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-oauth-api-key-and-basic-authentication workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-oauth-api-key-and-basic-authentication processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Identity and Authentication in Power Platform - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-oauth-api-key-and-basic-authentication protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-oauth-api-key-and-basic-authentication directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-oauth-api-key-and-basic-authentication execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("api-key-basic-auth", "api-key-header-security", "connector-credential-storage") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
