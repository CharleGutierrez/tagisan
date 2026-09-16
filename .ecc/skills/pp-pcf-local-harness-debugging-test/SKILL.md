---
name: pp-pcf-local-harness-debugging-test
description: "Testing PCF controls locally via npm start test harness, parameter mocking, and hot reload."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-25"]
triggers: ["local-harness-debugging", "pcf-test-harness", "hot-reload-pcf-debugging"]
---

# pp-pcf-local-harness-debugging-test

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Professional PCF Development: Extending Power Apps - Greg Hurlman'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pcf-local-harness-debugging-test workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pcf-local-harness-debugging-test processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Professional PCF Development: Extending Power Apps - Greg Hurlman*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pcf-local-harness-debugging-test protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pcf-local-harness-debugging-test directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pcf-local-harness-debugging-test execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("local-harness-debugging", "pcf-test-harness", "hot-reload-pcf-debugging") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
