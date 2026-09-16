---
name: pp-pweb-basic-forms-and-multistep-forms
description: "Configuring Basic Forms, Multistep Forms, session state persistence, and branch logic."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-19"]
triggers: ["basic-multistep-forms", "multistep-form-sessions", "form-branching-logic-pages"]
---

# pp-pweb-basic-forms-and-multistep-forms

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Secure Portal Development with Power Pages Web API - Colin Vermander'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pweb-basic-forms-and-multistep-forms workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pweb-basic-forms-and-multistep-forms processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Secure Portal Development with Power Pages Web API - Colin Vermander*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pweb-basic-forms-and-multistep-forms protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pweb-basic-forms-and-multistep-forms directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pweb-basic-forms-and-multistep-forms execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("basic-multistep-forms", "multistep-form-sessions", "form-branching-logic-pages") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
