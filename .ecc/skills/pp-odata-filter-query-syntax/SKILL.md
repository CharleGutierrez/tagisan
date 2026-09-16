---
name: pp-odata-filter-query-syntax
description: "Precision OData filter queries: eq, ne, gt, ge, lt, le, and, or, not, startswith, endswith."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-12"]
triggers: ["odata-filter-query", "odata-comparison-operators", "server-side-filter-syntax"]
---

# pp-odata-filter-query-syntax

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'OData and JSON Mastery for Power Platform Integrations - Microsoft Press'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-odata-filter-query-syntax workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-odata-filter-query-syntax processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *OData and JSON Mastery for Power Platform Integrations - Microsoft Press*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-odata-filter-query-syntax protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-odata-filter-query-syntax directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-odata-filter-query-syntax execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("odata-filter-query", "odata-comparison-operators", "server-side-filter-syntax") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
