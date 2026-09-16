---
name: o365-addin-core-shared-runtime-cross-component-state
description: "Configuring <Runtimes> for shared JS state between ribbon, taskpane, and functions."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-18"]
triggers: ["shared-runtime-state", "cross-component-js-state", "shared-runtime-office-addin"]
---

# o365-addin-core-shared-runtime-cross-component-state

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'The Modern Office Add-in Manifest: Migrating to JSON - Microsoft Learn'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-addin-core-shared-runtime-cross-component-state pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-addin-core-shared-runtime-cross-component-state operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *The Modern Office Add-in Manifest: Migrating to JSON - Microsoft Learn*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-addin-core-shared-runtime-cross-component-state protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-addin-core-shared-runtime-cross-component-state directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-addin-core-shared-runtime-cross-component-state execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("shared-runtime-state", "cross-component-js-state", "shared-runtime-office-addin") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
