---
name: pp-oauth-mtls-client-certificates
description: "Configuring Mutual TLS (mTLS) client certificate authentication for ultra-secure endpoints."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-29"]
triggers: ["mtls-client-certificates", "mutual-tls-connectors", "certificate-bound-apis"]
---

# pp-oauth-mtls-client-certificates

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Enterprise Identity and Authentication in Power Platform - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-oauth-mtls-client-certificates workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-oauth-mtls-client-certificates processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Enterprise Identity and Authentication in Power Platform - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-oauth-mtls-client-certificates protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-oauth-mtls-client-certificates directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-oauth-mtls-client-certificates execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("mtls-client-certificates", "mutual-tls-connectors", "certificate-bound-apis") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
