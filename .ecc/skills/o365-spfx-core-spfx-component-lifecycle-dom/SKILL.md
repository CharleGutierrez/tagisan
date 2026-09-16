---
name: o365-spfx-core-spfx-component-lifecycle-dom
description: "onInit(), render(), onDispose(), DOM container attachment, avoiding memory leaks."
version: 1.0.0
tier: "Enterprise / Microsoft Office 365 & Vibe Code Development"
tags: ["office-365", "vibe-coding", "cluster-11"]
triggers: ["spfx-component-lifecycle", "spfx-dom-attachment", "ondispose-cleanup-spfx"]
---

# o365-spfx-core-spfx-component-lifecycle-dom

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Engineering Standard: Conforms strictly to enterprise patterns established in 'Beginning SharePoint Framework Development - Vipul Jain & Andrew Connell'.
- Operational Directive: ALWAYS enforce deterministic execution boundaries and validate parameters before invoking o365-spfx-core-spfx-component-lifecycle-dom pipelines.
- Security Directive: NEVER bypass tenant authorization, least privilege scopes, or data boundary policies during o365-spfx-core-spfx-component-lifecycle-dom operations.
- Compliance Directive: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Beginning SharePoint Framework Development - Vipul Jain & Andrew Connell*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the o365-spfx-core-spfx-component-lifecycle-dom protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for o365-spfx-core-spfx-component-lifecycle-dom directly into production without staging verification.
- Silently ignoring transient failures or error codes during o365-spfx-core-spfx-component-lifecycle-dom execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("spfx-component-lifecycle", "spfx-dom-attachment", "ondispose-cleanup-spfx") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test office365_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
