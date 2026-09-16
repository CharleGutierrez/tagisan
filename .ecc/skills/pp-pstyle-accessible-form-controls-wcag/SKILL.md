---
name: pp-pstyle-accessible-form-controls-wcag
description: "Enforcing WCAG 2.1 AA accessible labels, focus rings, high contrast, and error announcements."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-20"]
triggers: ["accessible-form-controls-pages", "wcag-portal-compliance", "aria-error-announcements"]
---

# pp-pstyle-accessible-form-controls-wcag

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Styling and Theming Microsoft Power Pages - Ulrikke Akerbæk'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pstyle-accessible-form-controls-wcag workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pstyle-accessible-form-controls-wcag processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Styling and Theming Microsoft Power Pages - Ulrikke Akerbæk*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pstyle-accessible-form-controls-wcag protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pstyle-accessible-form-controls-wcag directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pstyle-accessible-form-controls-wcag execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("accessible-form-controls-pages", "wcag-portal-compliance", "aria-error-announcements") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
