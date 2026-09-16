---
name: pp-pcf-context-parameters-formatting
description: "Utilizing context.parameters for input bounds and context.formatting for localized dates and currencies."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-25"]
triggers: ["context-parameters-formatting", "localized-formatting-pcf", "context-parameters-api"]
---

# pp-pcf-context-parameters-formatting

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Professional PCF Development: Extending Power Apps - Greg Hurlman'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pcf-context-parameters-formatting workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pcf-context-parameters-formatting processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Professional PCF Development: Extending Power Apps - Greg Hurlman*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pcf-context-parameters-formatting protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pcf-context-parameters-formatting directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pcf-context-parameters-formatting execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("context-parameters-formatting", "localized-formatting-pcf", "context-parameters-api") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
