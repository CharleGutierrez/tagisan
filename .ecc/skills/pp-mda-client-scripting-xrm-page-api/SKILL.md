---
name: pp-mda-client-scripting-xrm-page-api
description: "Modern Xrm.WebApi client scripting, execution context lifecycle, and event registration."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-7"]
triggers: ["xrm-page-client-scripting", "xrm-webapi-crud", "form-execution-context"]
---

# pp-mda-client-scripting-xrm-page-api

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Mastering Model-Driven Apps in Power Apps - Phil Cole'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-mda-client-scripting-xrm-page-api workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-mda-client-scripting-xrm-page-api processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Mastering Model-Driven Apps in Power Apps - Phil Cole*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-mda-client-scripting-xrm-page-api protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-mda-client-scripting-xrm-page-api directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-mda-client-scripting-xrm-page-api execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("xrm-page-client-scripting", "xrm-webapi-crud", "form-execution-context") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
