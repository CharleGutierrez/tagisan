---
name: pp-conn-webhook-trigger-definitions
description: "Defining OpenAPI x-ms-trigger webhook definitions, subscription endpoints, and unsubscribe paths."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-28"]
triggers: ["webhook-trigger-definitions", "x-ms-trigger-webhooks", "subscription-lifecycle-connectors"]
---

# pp-conn-webhook-trigger-definitions

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Custom Connectors for Power Platform - Troy Taylor'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-conn-webhook-trigger-definitions workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-conn-webhook-trigger-definitions processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Custom Connectors for Power Platform - Troy Taylor*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-conn-webhook-trigger-definitions protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-conn-webhook-trigger-definitions directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-conn-webhook-trigger-definitions execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("webhook-trigger-definitions", "x-ms-trigger-webhooks", "subscription-lifecycle-connectors") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
