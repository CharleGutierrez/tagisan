---
name: pp-err-retry-policy-exponential-backoff
description: "Configuring exponential backoff and fixed interval retry policies for HTTP 429 throttling."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-10"]
triggers: ["retry-policy-backoff", "http-429-throttling-flow", "exponential-retry-tuning"]
---

# pp-err-retry-policy-exponential-backoff

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Bulletproof Cloud Flows: Enterprise Exception Handling - Pieter Veenstra'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-err-retry-policy-exponential-backoff workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-err-retry-policy-exponential-backoff processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Bulletproof Cloud Flows: Enterprise Exception Handling - Pieter Veenstra*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-err-retry-policy-exponential-backoff protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-err-retry-policy-exponential-backoff directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-err-retry-policy-exponential-backoff execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("retry-policy-backoff", "http-429-throttling-flow", "exponential-retry-tuning") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
