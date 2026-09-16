---
name: pp-odata-orderby-and-top-pagination
description: "Combining $orderby and $top parameters for predictable, server-side deterministic pagination."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-12"]
triggers: ["orderby-top-pagination", "odata-sorting-pagination", "deterministic-paging"]
---

# pp-odata-orderby-and-top-pagination

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'OData and JSON Mastery for Power Platform Integrations - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-odata-orderby-and-top-pagination workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-odata-orderby-and-top-pagination processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *OData and JSON Mastery for Power Platform Integrations - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-odata-orderby-and-top-pagination protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-odata-orderby-and-top-pagination directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-odata-orderby-and-top-pagination execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("orderby-top-pagination", "odata-sorting-pagination", "deterministic-paging") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
