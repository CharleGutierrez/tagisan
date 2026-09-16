---
name: pp-conn-x-ms-summary-visibility-metadata
description: "Annotating endpoints with x-ms-summary and x-ms-visibility (important, advanced, internal)."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-28"]
triggers: ["x-ms-summary-visibility", "connector-metadata-annotations", "x-ms-visibility-levels"]
---

# pp-conn-x-ms-summary-visibility-metadata

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Building Custom Connectors for Power Platform - Troy Taylor'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-conn-x-ms-summary-visibility-metadata workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-conn-x-ms-summary-visibility-metadata processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Building Custom Connectors for Power Platform - Troy Taylor*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-conn-x-ms-summary-visibility-metadata protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-conn-x-ms-summary-visibility-metadata directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-conn-x-ms-summary-visibility-metadata execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("x-ms-summary-visibility", "connector-metadata-annotations", "x-ms-visibility-levels") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
