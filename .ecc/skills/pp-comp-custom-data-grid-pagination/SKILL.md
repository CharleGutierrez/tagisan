---
name: pp-comp-custom-data-grid-pagination
description: "Custom canvas grid with column sorting indicators, virtual scrolling, and page size controls."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-3"]
triggers: ["custom-data-grid", "canvas-grid-pagination", "sorting-table-component"]
---

# pp-comp-custom-data-grid-pagination

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Reusable Canvas Components and Design Systems - Hardit Bhatia'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-comp-custom-data-grid-pagination workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-comp-custom-data-grid-pagination processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Reusable Canvas Components and Design Systems - Hardit Bhatia*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-comp-custom-data-grid-pagination protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-comp-custom-data-grid-pagination directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-comp-custom-data-grid-pagination execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("custom-data-grid", "canvas-grid-pagination", "sorting-table-component") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
