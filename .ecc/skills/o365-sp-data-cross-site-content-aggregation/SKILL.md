---
name: o365-sp-data-cross-site-content-aggregation
description: "Executing KQL search queries via REST to aggregate items across hundreds of modern sites."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-14"]
triggers: ["cross-site-content-aggregation", "rest-kql-search-aggregation", "enterprise-task-rollup"]
---

# o365-sp-data-cross-site-content-aggregation

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'SharePoint REST API and OData Programming - Gary Lapointe & Patrick Rodgers'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-sp-data-cross-site-content-aggregation pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-sp-data-cross-site-content-aggregation operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *SharePoint REST API and OData Programming - Gary Lapointe & Patrick Rodgers*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-sp-data-cross-site-content-aggregation protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-sp-data-cross-site-content-aggregation directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-sp-data-cross-site-content-aggregation execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("cross-site-content-aggregation", "rest-kql-search-aggregation", "enterprise-task-rollup") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
