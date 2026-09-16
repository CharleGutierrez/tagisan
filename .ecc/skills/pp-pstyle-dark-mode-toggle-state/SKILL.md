---
name: pp-pstyle-dark-mode-toggle-state
description: "Implementing CSS prefers-color-scheme media queries and localStorage dark mode toggle switches."
version: 1.0.0
tier: "Enterprise / Microsoft Power Platform & Vibe Code Development"
tags: ["power-platform", "vibe-coding", "cluster-20"]
triggers: ["dark-mode-toggle-state", "prefers-color-scheme-pages", "localstorage-theme-toggle"]
---

# pp-pstyle-dark-mode-toggle-state

## 1. Core Mathematical & Architectural Foundations / Formal Invariants
- Architecture Standard: Conforms strictly to enterprise patterns defined in 'Styling and Theming Microsoft Power Pages - Ulrikke Akerbæk'.
- Operational Invariant: ALWAYS enforce deterministic execution boundaries and validate inputs before invoking pp-pstyle-dark-mode-toggle-state workflows.
- Security Directive: NEVER bypass enterprise authorization, encryption, or DLP policies during pp-pstyle-dark-mode-toggle-state processing.
- Compliance Rule: MANDATORY validation of schema integrity, transactional consistency, and error traps across all integration steps.
- Source Reference: *Styling and Theming Microsoft Power Pages - Ulrikke Akerbæk*

## 2. Concrete Agent Specification & Prompt Contract (Vibe Coder Protocol)
ALWAYS apply the pp-pstyle-dark-mode-toggle-state protocol according to enterprise standards. Structure input payloads with explicit schemas, assert preconditions, execute operations within bounded timeouts, and log complete diagnostic telemetry.

## 3. Anti-Patterns & Hallucination Mitigations for AI Coding Agents
- Deploying unvalidated configurations for pp-pstyle-dark-mode-toggle-state directly into production without staging verification.
- Silently ignoring transient failures or error codes during pp-pstyle-dark-mode-toggle-state execution.

## 4. Executable Verification Recipe
1. Validate frontmatter conforms to RFC-004 standard (name, description, version, tier, tags, triggers).
2. Ensure triggers ("dark-mode-toggle-state", "prefers-color-scheme-pages", "localstorage-theme-toggle") are correctly indexed by Tagisan ECC dispatcher.
3. Verify that operational directives (ALWAYS, NEVER, MANDATORY) are strictly enforced in generated code.
4. Execute `cargo test --test power_platform_skills_brutal_tests` to verify memory safety, zero-panic invariants, and concurrent dispatching.
