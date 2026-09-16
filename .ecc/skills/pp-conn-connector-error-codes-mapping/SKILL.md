---
name: pp-conn-connector-error-codes-mapping
description: "Mapping HTTP 4xx/5xx status codes into user-friendly error messages and troubleshooting hints."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-28"]
triggers: ["connector-error-codes-mapping", "http-error-normalization", "user-friendly-api-errors"]
---

# pp-conn-connector-error-codes-mapping

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Custom Connectors for Power Platform - Troy Taylor'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-conn-connector-error-codes-mapping workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-conn-connector-error-codes-mapping processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Custom Connectors for Power Platform - Troy Taylor*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-conn-connector-error-codes-mapping protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-conn-connector-error-codes-mapping directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-conn-connector-error-codes-mapping execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("connector-error-codes-mapping", "http-error-normalization", "user-friendly-api-errors") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
